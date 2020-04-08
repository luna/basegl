//! FRP keyboard bindings.

use crate::prelude::*;

use rust_dense_bitset::BitSet;
use rust_dense_bitset::DenseBitSetExtended;
use std::collections::hash_map::Entry;
use crate as frp;



// ===========
// === Key ===
// ===========

/// A key representation.
pub use keyboard_types::Key;



// ===============
// === KeyMask ===
// ===============

/// The maximum possible key code used as size of `KeyMask` bitset.
const MAX_KEY_CODE : usize = 255;

/// The key bitmask (each bit represents one key). Used for matching key combinations.
#[derive(BitXor,Clone,Debug,Eq,Hash,PartialEq,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct KeyMask(pub DenseBitSetExtended);

impl KeyMask {
    /// Creates Key::Control + Key::Character.
    pub fn new_control_character(character:char) -> Self {
        Self::from_vec(vec![Key::Control, Key::Character(character.to_string())])
    }

    /// Creates Key::Alt + Key::Character.
    pub fn new_alt_character(character:char) -> Self {
        Self::from_vec(vec![Key::Alt, Key::Character(character.to_string())])
    }

    /// Creates KeyMask from Vec<Key>.
    pub fn from_vec(keys:Vec<Key>) -> Self {
        keys.iter().collect()
    }

    /// Check if key bit is on.
    pub fn has_key(&self, key:&Key) -> bool {
        let KeyMask(bit_set) = self;
        bit_set.get_bit(Self::bit_position_for(key))
    }

    /// Set the `key` bit for new state.
    pub fn set_key(&mut self, key:&Key, state:bool) {
        let KeyMask(ref mut bit_set) = self;
        bit_set.set_bit(Self::bit_position_for(key),state);
    }
}

impl Default for KeyMask {
    fn default() -> Self {
        let mut bitset = DenseBitSetExtended::with_capacity(MAX_KEY_CODE + 1);
        // This is the only way to set bitset length.
        bitset.set_bit(MAX_KEY_CODE,true);
        bitset.set_bit(MAX_KEY_CODE,false);
        Self(bitset)
    }
}

impl<'a> FromIterator<&'a Key> for KeyMask {
    fn from_iter<T: IntoIterator<Item=&'a Key>>(iter:T) -> Self {
        let mut key_mask = KeyMask::default();
        for key in iter {
            let bit = Self::bit_position_for(key);
            key_mask.set_bit(bit,true);
        }
        key_mask
    }
}

impl From<&[Key]> for KeyMask {
    fn from(keys: &[Key]) -> Self {
        <KeyMask as FromIterator<&Key>>::from_iter(keys)
    }
}


// === Private ===

impl KeyMask {
    fn bit_position_for(key:&Key) -> usize {
        (match key {
            // On Chrome, shift+alt gives `Meta` key, therefore we do a unification here.
            Key::Meta => Key::Alt.legacy_keycode(),
            other     => other.legacy_keycode(),
        }) as usize
    }
}



// ================
// === KeyState ===
// ================

/// A helper structure used for describing KeyMask changes.
#[derive(Clone,Debug)]
enum KeyMaskChange {
    Set(Key),
    Unset(Key),
    Clear,
}

impl KeyMaskChange {
    fn on_pressed  (key:&Key) -> Self {
        Self::Set(key.clone())
    }

    /// When we're losing focus we should clear keymask, because we are not sure what keys were
    /// released during being unfocused.
    fn on_defocus() -> Self {
        Self::Clear
    }

    fn on_released (key:&Key) -> Self {
        match key {
            // The very special case: pressing CMD on MacOS makes all the keyup events for letters
            // lost. Therefore for CMD releasing we must clear keymask.
            Key::Meta => Self::Clear,
            other     => Self::Unset(other.clone())
        }
    }

    /// Returns copy of given KeyMask with applied change
    fn updated_mask(&self, mask:&KeyMask) -> KeyMask {
        let mut mask = mask.clone();
        match self {
            Self::Set(ref key)   => mask.set_key(key, true),
            Self::Unset(ref key) => mask.set_key(key, false),
            Self::Clear          => mask = default()
        }
        mask
    }
}

impl Default for KeyMaskChange {
    fn default() -> Self {
        Self::Clear
    }
}



// ================
// === Keyboard ===
// ================

/// Keyboard FRP bindings.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Keyboard {
    pub network     : frp::Network,
    pub on_pressed  : frp::Source<Key>,
    pub on_released : frp::Source<Key>,
    pub on_defocus  : frp::Source,
    pub key_mask    : frp::Stream<KeyMask>,
}

impl Default for Keyboard {
    fn default() -> Self {
        frp::new_network! { keyboard
            def on_pressed        = source();
            def on_released       = source();
            def on_defocus        = source();
            def change_set        = on_pressed.map(KeyMaskChange::on_pressed);
            def change_unset      = on_released.map(KeyMaskChange::on_released);
            def change_clear      = on_defocus.map(|()| KeyMaskChange::on_defocus());
            def change_set_unset  = change_set.merge(&change_unset);
            def change            = change_set_unset.merge(&change_clear);
            def previous_key_mask = gather::<KeyMask>();
            def key_mask          = change.map2(&previous_key_mask,KeyMaskChange::updated_mask);
        }
        previous_key_mask.add(&key_mask);
        let network = keyboard;
        Keyboard {network,on_pressed,on_released,on_defocus,key_mask}
    }
}



// =======================
// === KeyboardActions ===
// =======================

/// An action defined for specific key combinations. For convenience, the key mask is passed as
/// argument.
pub trait Action = FnMut(&KeyMask) + 'static;

/// A mapping between key combinations and actions.
pub type ActionMap = HashMap<KeyMask,Box<dyn Action>>;

/// A structure bound to Keyboard FRP graph, which allows to define actions for specific keystrokes.
pub struct KeyboardActions {
    action_map : Rc<RefCell<ActionMap>>,
    _network   : frp::Network,
    _action    : frp::Stream,
}

impl KeyboardActions {
    /// Create structure without any actions defined yet. It will be listening for events from
    /// passed `Keyboard` structure.
    pub fn new(keyboard:&Keyboard) -> Self {
        let action_map = Rc::new(RefCell::new(HashMap::new()));
        frp::new_network! { keyboard_actions
            def action = keyboard.key_mask.map(Self::perform_action_lambda(action_map.clone()));
        }
        let _network = keyboard_actions;
        let _action  = action;
        KeyboardActions{action_map,_network,_action}
    }

    fn perform_action_lambda(action_map:Rc<RefCell<ActionMap>>) -> impl Fn(&KeyMask) {
        move |key_mask| {
            let entry_opt = with(action_map.borrow_mut(), |mut map| map.remove_entry(key_mask));
            if let Some((map_mask, mut action)) = entry_opt {
                action(key_mask);
                if let Entry::Vacant(entry) =  action_map.borrow_mut().entry(map_mask) {
                    entry.insert(action);
                }
            }
        }
    }

    /// Set action binding for given key mask.
    pub fn set_action<F:FnMut(&KeyMask) + 'static>(&mut self, key_mask:KeyMask, action:F) {
        self.action_map.borrow_mut().insert(key_mask,Box::new(action));
    }

    /// Remove action binding for given key mask.
    pub fn unset_action(&mut self, key_mask:&KeyMask) {
        self.action_map.borrow_mut().remove(key_mask);
    }
}

impl Debug for KeyboardActions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<KeyboardActions>")
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_mask() {
        let keyboard                  = Keyboard::default();
        let expected_key_mask:KeyMask = default();
        assert_eq!(expected_key_mask, keyboard.key_mask.value());
        let key1 = Key::Character("x".to_string());
        let key2 = Key::Control;

        keyboard.on_pressed.emit(key1.clone());
        let expected_key_mask:KeyMask = std::iter::once(&key1).collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.value());

        keyboard.on_pressed.emit(key2.clone());
        let expected_key_mask:KeyMask = [&key1,&key2].iter().cloned().collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.value());

        keyboard.on_released.emit(key1.clone());
        let expected_key_mask:KeyMask = std::iter::once(&key2).collect();
        assert_eq!(expected_key_mask, keyboard.key_mask.value());
    }

    #[test]
    fn key_actions() {
        use keyboard_types::Key::*;
        let undone            = Rc::new(RefCell::new(false));
        let undone1           = undone.clone();
        let redone            = Rc::new(RefCell::new(false));
        let redone1           = redone.clone();
        let undo_keys:KeyMask = [Control, Character("z".to_string())].iter().collect();
        let redo_keys:KeyMask = [Control, Character("y".to_string())].iter().collect();

        let keyboard    = Keyboard::default();
        let mut actions = KeyboardActions::new(&keyboard);
        actions.set_action(undo_keys.clone(), move |_| { *undone1.borrow_mut() = true });
        actions.set_action(redo_keys.clone(), move |_| { *redone1.borrow_mut() = true });
        keyboard.on_pressed.emit(Character("Z".to_string()));
        assert!(!*undone.borrow());
        assert!(!*redone.borrow());
        keyboard.on_pressed.emit(Control);
        assert!( *undone.borrow());
        assert!(!*redone.borrow());
        *undone.borrow_mut() = false;
        keyboard.on_released.emit(Character("z".to_string()));
        assert!(!*undone.borrow());
        assert!(!*redone.borrow());
        keyboard.on_pressed.emit(Character("y".to_string()));
        assert!(!*undone.borrow());
        assert!( *redone.borrow());
        *redone.borrow_mut() = false;
        keyboard.on_released.emit(Character("y".to_string()));
        keyboard.on_released.emit(Control);

        actions.unset_action(&undo_keys);
        keyboard.on_pressed.emit(Character("Z".to_string()));
        keyboard.on_pressed.emit(Control);
        assert!(!*undone.borrow());
        assert!(!*redone.borrow());
    }
}