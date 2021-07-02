use crate::prelude::*;

use enso_protocol::language_server;
use enso_frp as frp;
use enso_protocol::language_server::FileSystemObject;
use ensogl_gui_components::file_browser::model::FolderContent;
use ensogl_gui_components::file_browser::model::FolderType;
use ensogl_gui_components::file_browser::model::Entry;
use ensogl_gui_components::file_browser::model::EntryType;


#[derive(Clone,Debug)]
pub struct FileProvider {
    pub connection    : Rc<language_server::Connection>,
    pub content_roots : Vec<Rc<language_server::types::ContentRoot>>,
}

impl FolderContent for FileProvider {
    fn request_entries
    (&self, entries_loaded:frp::Any<Rc<Vec<Entry>>>, _error_occurred:frp::Any<ImString>) {
        let entries = self.content_roots.iter().map(|root| {
            let folder_type = match root.content_root_type {
                language_server::ContentRootType::Project => FolderType::Project,
                language_server::ContentRootType::Root    => FolderType::Root,
                language_server::ContentRootType::Home    => FolderType::Home,
                language_server::ContentRootType::Library => FolderType::Library,
                language_server::ContentRootType::Custom  => FolderType::Custom,
            };
            Entry {
                name: root.name.clone(),
                path: root.id.to_string().into(),
                type_: EntryType::Folder {
                    type_ : folder_type,
                    content: {
                        let connection = self.connection.clone_ref();
                        DirectoryView::new_from_root(connection,root.clone_ref()).into()
                    }
                }
            }
        });
        entries_loaded.emit(Rc::new(entries.collect_vec()));
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct DirectoryView {
    connection   : Rc<language_server::Connection>,
    content_root : Rc<language_server::types::ContentRoot>,
    path         : Rc<language_server::Path>,
}

impl DirectoryView {
    fn new_from_root
    ( connection   : Rc<language_server::Connection>
    , content_root : Rc<language_server::types::ContentRoot>
    ) -> Self {
        let path = Rc::new(language_server::Path::new_root(content_root.id));
        Self{connection,content_root,path}
    }

    fn sub_view(&self, segment:impl Str) -> DirectoryView {
        DirectoryView {
            connection   : self.connection.clone_ref(),
            content_root : self.content_root.clone_ref(),
            path         : Rc::new(self.path.append_im(segment))
        }
    }

    async fn get_entries_list(&self) -> FallibleResult<Vec<Entry>> {
        let response = self.connection.file_list(&self.path).await?;
        let entries  = response.paths.into_iter().map(|fs_obj| {
            match fs_obj {
                FileSystemObject::Directory {name,path} |
                FileSystemObject::DirectoryTruncated {name,path} |
                FileSystemObject::SymlinkLoop {name,path,..} => {
                    let path  = path.to_string().into();
                    let sub   = self.sub_view(&name);
                    let type_ = EntryType::Folder {
                        type_   : FolderType::Standard,
                        content : sub.into()
                    };
                    Entry {name,path,type_}
                }
                FileSystemObject::File {name,path}  |
                FileSystemObject::Other {name,path} => {
                    let path  = path.to_string().into();
                    let type_ = EntryType::File;
                    Entry {name,path,type_}
                }
            }
        });
        Ok(entries.collect())
    }
}

impl FolderContent for DirectoryView {
    fn request_entries
    (&self, entries_loaded:frp::Any<Rc<Vec<Entry>>>, error_occurred:frp::Any<ImString>) {
        let this = self.clone_ref();
        executor::global::spawn(async move {
            match this.get_entries_list().await {
                Ok (entries) => entries_loaded.emit(Rc::new(entries)),
                Err(error)   => error_occurred.emit(ImString::new(error.to_string())),
            }
        });
    }
}
