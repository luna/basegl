#![allow(missing_docs)]

use crate::prelude::*;
use crate::stream::*;
use crate::stream;
use crate::debug;
use crate::network::*;
use crate::node::*;



pub fn watch_stream<T:StreamOutput>(target:&T) -> Watched<T> {
    let target = target.clone_ref();
    let handle = target.register_watch();
    Watched::new(target,handle)
}



// ==============
// === Source ===
// ==============

macro_rules! docs_for_source { ($($tt:tt)*) => { #[doc="
Begin point in the FRP network. It does not accept inputs, but it is able to emit events. Often it
is used to indicate that something happened, like a button was pressed. In such cases its type
parameter is set to an empty tuple.
"]$($tt)* }}

docs_for_source! { #[derive(Clone,Copy,Debug)]
pub struct SourceData <Out=()> { phantom:PhantomData<Out> }}
pub type   Source     <Out=()> = StreamNode     <SourceData<Out>>;
pub type   WeakSource <Out=()> = WeakStreamNode <SourceData<Out>>;

impl<Out:Data> HasOutput for SourceData<Out> {
    type Output = Out;
}

impl<Out:Data> Source<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom    = default();
        let definition = SourceData {phantom};
        Self::construct(label,definition)
    }
}



// =============
// === Trace ===
// =============

macro_rules! docs_for_trace { ($($tt:tt)*) => { #[doc="
Print the incoming events to console and pass them to output.
"]$($tt)* }}

/// Print the incoming events to console and pass them to output.
#[derive(Clone,Debug)]
pub struct TraceData <Out> { phantom:PhantomData<Out>, message:String }
pub type   Trace     <Out> = StreamNode     <TraceData<Out>>;
pub type   WeakTrace <Out> = WeakStreamNode <TraceData<Out>>;

impl<Out:Data> HasOutput for TraceData<Out> {
    type Output = Out;
}

impl<Out:Data> Trace<Out> {
    /// Constructor.
    pub fn new<M,S>(label:Label, message:M, stream:&S) -> Self
    where M:Into<String>, S:StreamOutput<Output=Out> {
        let phantom = default();
        let message = message.into();
        let def     = TraceData {phantom,message};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<Out:Data> EventConsumer<Out> for Trace<Out> {
    fn on_event(&self, event:&Out) {
        println!("[FRP] {}: {:?}", self.message, event);
        self.emit(event);
    }
}



// ==============
// === Toggle ===
// ==============

macro_rules! docs_for_toggle { ($($tt:tt)*) => { #[doc="
Emits `true`, `false`, `true`, `false`, ... on every incoming event.
"]$($tt)* }}

docs_for_toggle! { #[derive(Clone,Debug)]
pub struct ToggleData { value:Cell<bool> }}
pub type   Toggle     = StreamNode     <ToggleData>;
pub type   WeakToggle = WeakStreamNode <ToggleData>;

impl HasOutput for ToggleData {
    type Output = bool;
}

impl Toggle {
    /// Constructor.
    pub fn new<S:StreamOutput>(label:Label, stream:&S) -> Self {
        Self::new_with(label,stream,default())
    }

    /// Constructor with explicit start value.
    pub fn new_with<S:StreamOutput>(label:Label, stream:&S, init:bool) -> Self {
        let value = Cell::new(init);
        let def   = ToggleData {value};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<T> EventConsumer<T> for Toggle {
    fn on_event(&self, _:&T) {
        let value = !self.value.get();
        self.value.set(value);
        self.emit(value);
    }
}



// =============
// === Count ===
// =============

macro_rules! docs_for_count { ($($tt:tt)*) => { #[doc="
Count the incoming events.
"]$($tt)* }}

docs_for_count! { #[derive(Debug)]
pub struct CountData { value:Cell<usize> }}
pub type   Count     = StreamNode     <CountData>;
pub type   WeakCount = WeakStreamNode <CountData>;

impl HasOutput for CountData {
    type Output = usize;
}

impl Count {
    /// Constructor.
    pub fn new<S>(label:Label, stream:&S) -> Self
    where S:StreamOutput {
        let value = default();
        let def   = CountData {value};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<T> EventConsumer<T> for Count {
    fn on_event(&self, _:&T) {
        let value = self.value.get() + 1;
        self.value.set(value);
        self.emit(value);
    }
}



// ================
// === Constant ===
// ================

macro_rules! docs_for_constant { ($($tt:tt)*) => { #[doc="
Replaces the incoming event with the predefined value.
"]$($tt)* }}

docs_for_constant! { #[derive(Clone,Debug)]
pub struct ConstantData <Out=()> { value:Out }}
pub type   Constant     <Out=()> = StreamNode     <ConstantData<Out>>;
pub type   WeakConstant <Out=()> = WeakStreamNode <ConstantData<Out>>;

impl<Out:Data> HasOutput for ConstantData<Out> {
    type Output = Out;
}

impl<Out:Data> Constant<Out> {
    /// Constructor.
    pub fn new<S>(label:Label, stream:&S, value:Out) -> Self
    where S:StreamOutput {
        let def = ConstantData {value};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<Out:Data,T> EventConsumer<T> for Constant<Out> {
    fn on_event(&self, _:&T) {
        self.emit(&self.value);
    }
}



// ================
// === Previous ===
// ================

macro_rules! docs_for_previous { ($($tt:tt)*) => { #[doc="
Remembers the value of the input stream and outputs the previously received one.
"]$($tt)* }}

docs_for_previous! { #[derive(Clone,Debug)]
pub struct PreviousData <Out=()> { previous:RefCell<Out> }}
pub type   Previous     <Out=()> = StreamNode     <PreviousData<Out>>;
pub type   WeakPrevious <Out=()> = WeakStreamNode <PreviousData<Out>>;

impl<Out:Data> HasOutput for PreviousData<Out> {
    type Output = Out;
}

impl<Out:Data> Previous<Out> {
    /// Constructor.
    pub fn new<S>(label:Label, stream:&S) -> Self
        where S:StreamOutput<Output=Out> {
        let previous = default();
        let def      = PreviousData {previous};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<Out:Data> EventConsumer<Out> for Previous<Out> {
    fn on_event(&self, event:&Out) {
        let previous = mem::replace(&mut *self.previous.borrow_mut(),event.clone());
        self.emit(previous);
    }
}



// ==============
// === Sample ===
// ==============

macro_rules! docs_for_sample { ($($tt:tt)*) => { #[doc="
Samples the first stream (behavior) on every incoming event of the second stream. The incoming event
is dropped and a new event with the behavior's value is emitted.
"]$($tt)* }}

docs_for_sample! { #[derive(Debug)]
pub struct SampleData <T1> { behavior:Watched<T1> }}
pub type   Sample     <T1> = StreamNode     <SampleData<T1>>;
pub type   WeakSample <T1> = WeakStreamNode <SampleData<T1>>;

impl<T1:HasOutput> HasOutput for SampleData<T1> {
    type Output = Output<T1>;
}

impl<T1:StreamOutput> Sample<T1> {
    /// Constructor.
    pub fn new<Event:StreamOutput>(label:Label, event:&Event, behavior:&T1) -> Self {
        let behavior   = watch_stream(behavior);
        let definition = SampleData {behavior};
        Self::construct_and_connect(label,event,definition)
    }
}

impl<T,T1:StreamOutput> EventConsumer<T> for Sample<T1> {
    fn on_event(&self, _:&T) {
        self.emit(self.behavior.value());
    }
}

impl<B> InputBehaviors for SampleData<B>
where B:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.behavior)]
    }
}



// ============
// === Gate ===
// ============

macro_rules! docs_for_gate { ($($tt:tt)*) => { #[doc="
Passes the incoming event of the fisr stream only if the value of the second stream is `true`.
"]$($tt)* }}

docs_for_gate! { #[derive(Debug)]
pub struct GateData <T1,Out=()> { behavior:Watched<T1>, phantom:PhantomData<Out> }}
pub type   Gate     <T1,Out=()> = StreamNode     <GateData<T1,Out>>;
pub type   WeakGate <T1,Out=()> = WeakStreamNode <GateData<T1,Out>>;

impl<T1,Out:Data> HasOutput for GateData<T1,Out> {
    type Output = Out;
}

impl<T1,Out> Gate<T1,Out>
where Out:Data, T1:StreamOutput<Output=bool> {
    /// Constructor.
    pub fn new<E>(label:Label, event:&E, behavior:&T1) -> Self
    where E:StreamOutput<Output=Out> {
        let behavior   = watch_stream(behavior);
        let phantom    = default();
        let definition = GateData {behavior,phantom};
        Self::construct_and_connect(label,event,definition)
    }
}

impl<T1,Out> EventConsumer<Out> for Gate<T1,Out>
where Out:Data, T1:StreamOutput<Output=bool> {
    fn on_event(&self, event:&Out) {
        if self.behavior.value() {
            self.emit(event)
        }
    }
}

impl<T1,Out> InputBehaviors for GateData<T1,Out>
where T1:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.behavior)]
    }
}



// =============
// === Merge ===
// =============

macro_rules! docs_for_merge { ($($tt:tt)*) => { #[doc="
Merges multiple input streams into a single output stream. All input streams have to share the same
output data type. Please note that `Merge` can be used to create recursive FRP networks by creating
an empty merge and using the `add` method to attach new streams to it. When a recursive network is
created, `Merge` breaks the cycle. After passing the first event, no more events will be passed
till the end of the current FRP network resolution.
"]$($tt)* }}

docs_for_merge! { #[derive(Clone,Debug)]
pub struct MergeData <Out=()> { phantom:PhantomData<Out>, during_call:Cell<bool> }}
pub type   Merge     <Out=()> = StreamNode     <MergeData<Out>>;
pub type   WeakMerge <Out=()> = WeakStreamNode <MergeData<Out>>;

impl<Out:Data> HasOutput for MergeData<Out> {
    type Output = Out;
}

impl<Out:Data> Merge<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom     = default();
        let during_call = default();
        let def         = MergeData {phantom,during_call};
        Self::construct(label,def)
    }

    /// Takes ownership of self and returns it with a new stream attached.
    pub fn with<S>(self, stream:&S) -> Self
        where S:StreamOutput<Output=Out> {
        stream.register_target(self.downgrade().into());
        self
    }

    /// Constructor for 1 input stream.
    pub fn new1<T1>(label:Label, t1:&T1) -> Self
        where T1:StreamOutput<Output=Out> {
        Self::new(label).with(t1)
    }

    /// Constructor for 2 input streams.
    pub fn new2<T1,T2>(label:Label, t1:&T1, t2:&T2) -> Self
        where T1:StreamOutput<Output=Out>,
              T2:StreamOutput<Output=Out> {
        Self::new(label).with(t1).with(t2)
    }

    /// Constructor for 3 input streams.
    pub fn new3<T1,T2,T3>(label:Label, t1:&T1, t2:&T2, t3:&T3) -> Self
        where T1:StreamOutput<Output=Out>,
              T2:StreamOutput<Output=Out>,
              T3:StreamOutput<Output=Out> {
        Self::new(label).with(t1).with(t2).with(t3)
    }

    /// Constructor for 4 input streams.
    pub fn new4<T1,T2,T3,T4>(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4) -> Self
        where T1:StreamOutput<Output=Out>,
              T2:StreamOutput<Output=Out>,
              T3:StreamOutput<Output=Out>,
              T4:StreamOutput<Output=Out> {
        Self::new(label).with(t1).with(t2).with(t3).with(t4)
    }
}

impl<Out:Data> WeakMerge<Out> {
    /// Takes ownership of self and returns it with a new stream attached.
    pub fn with<S>(self, stream:&S) -> Self
    where S:StreamOutput<Output=Out> {
        stream.register_target(self.clone_ref().into());
        self
    }
}

impl<T1,Out> Add<&T1> for &Merge<Out>
    where T1:StreamOutput<Output=Out>, Out:Data {
    type Output = Self;
    fn add(self, stream:&T1) -> Self::Output {
        stream.register_target(self.downgrade().into());
        self
    }
}

impl<T1,Out> Add<&T1> for &WeakMerge<Out>
    where T1:StreamOutput<Output=Out>, Out:Data {
    type Output = Self;
    fn add(self, stream:&T1) -> Self::Output {
        stream.register_target(self.into());
        self
    }
}

impl<Out:Data> EventConsumer<Out> for Merge<Out> {
    fn on_event(&self, event:&Out) {
        self.emit(event);
    }
}



// ============
// === Zip2 ===
// ============

macro_rules! docs_for_zip2 { ($($tt:tt)*) => { #[doc="
Merges two input streams into a stream containing values from both of them. On event from any of the
streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip2! { #[derive(Debug)]
pub struct Zip2Data <T1,T2> { stream1:Watched<T1>, stream2:Watched<T2> }}
pub type   Zip2     <T1,T2> = StreamNode     <Zip2Data<T1,T2>>;
pub type   WeakZip2 <T1,T2> = WeakStreamNode <Zip2Data<T1,T2>>;

impl<T1,T2> HasOutput for Zip2Data<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    type Output = (Output<T1>,Output<T2>);
}

impl<T1,T2> Zip2<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let def   = Zip2Data {stream1,stream2};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.into());
        this
    }
}

impl<T1,T2,Out> EventConsumer<Out> for Zip2<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    fn on_event(&self, _:&Out) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        self.emit((value1,value2));
    }
}

impl<T1,T2> InputBehaviors for Zip2Data<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::mixed(&self.stream1), Link::mixed(&self.stream2)]
    }
}



// ============
// === Zip3 ===
// ============

macro_rules! docs_for_zip3 { ($($tt:tt)*) => { #[doc="
Merges three input streams into a stream containing values from all of them. On event from any of
the streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip3! { #[derive(Debug)]
pub struct Zip3Data <T1,T2,T3> { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3> }}
pub type   Zip3     <T1,T2,T3> = StreamNode     <Zip3Data<T1,T2,T3>>;
pub type   WeakZip3 <T1,T2,T3> = WeakStreamNode <Zip3Data<T1,T2,T3>>;

impl<T1,T2,T3> HasOutput for Zip3Data<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    type Output = (Output<T1>,Output<T2>,Output<T3>);
}

impl<T1,T2,T3> Zip3<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let def   = Zip3Data {stream1,stream2,stream3};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,Out> EventConsumer<Out> for Zip3<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    fn on_event(&self, _:&Out) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        self.emit((value1,value2,value3));
    }
}

impl<T1,T2,T3> InputBehaviors for Zip3Data<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::mixed(&self.stream1), Link::mixed(&self.stream2), Link::mixed(&self.stream3)]
    }
}



// ============
// === Zip4 ===
// ============

macro_rules! docs_for_zip4 { ($($tt:tt)*) => { #[doc="
Merges four input streams into a stream containing values from all of them. On event from any of the
streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip4! { #[derive(Debug)]
pub struct Zip4Data <T1,T2,T3,T4>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, stream4:Watched<T4> }}
pub type   Zip4     <T1,T2,T3,T4> = StreamNode     <Zip4Data<T1,T2,T3,T4>>;
pub type   WeakZip4 <T1,T2,T3,T4> = WeakStreamNode <Zip4Data<T1,T2,T3,T4>>;

impl<T1,T2,T3,T4> HasOutput for Zip4Data<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    type Output = (Output<T1>,Output<T2>,Output<T3>,Output<T4>);
}

impl<T1,T2,T3,T4> Zip4<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let stream4 = watch_stream(t4);
        let def   = Zip4Data {stream1,stream2,stream3,stream4};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.clone_ref().into());
        t4.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,T4,Out> EventConsumer<Out> for Zip4<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    fn on_event(&self, _:&Out) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let value4 = self.stream4.value();
        self.emit((value1,value2,value3,value4));
    }
}

impl<T1,T2,T3,T4> InputBehaviors for Zip4Data<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![ Link::mixed(&self.stream1)
            , Link::mixed(&self.stream2)
            , Link::mixed(&self.stream3)
            , Link::mixed(&self.stream4)
            ]
    }
}



// ===========
// === Map ===
// ===========

macro_rules! docs_for_map { ($($tt:tt)*) => { #[doc="
On every event from the first input stream, sample all other input streams and run the provided
function on all gathered values. If you want to run the function on event from any input stream,
use the `apply` function family instead.
"]$($tt)* }}

docs_for_map! {
pub struct MapData <T1,F> { stream:Watched<T1>, function:F }}
pub type   Map     <T1,F> = StreamNode     <MapData<T1,F>>;
pub type   WeakMap <T1,F> = WeakStreamNode <MapData<T1,F>>;

impl<T1,F,Out> HasOutput for MapData<T1,F>
where T1:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>)->Out {
    type Output = Out;
}

impl<T1,F,Out> Map<T1,F>
where T1:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, function:F) -> Self {
        let stream     = watch_stream(t1);
        let definition = MapData {stream,function};
        Self::construct_and_connect(label,t1,definition)
    }
}

impl<T1,F,Out> EventConsumer<Output<T1>> for Map<T1,F>
where T1:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>)->Out {
    fn on_event(&self, value:&Output<T1>) {
        let out = (self.function)(value);
        self.emit(out);
    }
}

impl<T1,F> Debug for MapData<T1,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"MapData")
    }
}



// ============
// === Map2 ===
// ============

docs_for_map! {
pub struct Map2Data <T1,T2,F> { stream1:Watched<T1>, stream2:Watched<T2>, function:F }}
pub type   Map2     <T1,T2,F> = StreamNode     <Map2Data<T1,T2,F>>;
pub type   WeakMap2 <T1,T2,F> = WeakStreamNode <Map2Data<T1,T2,F>>;

impl<T1,T2,F,Out> HasOutput for Map2Data<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    type Output = Out;
}

impl<T1,T2,F,Out> Map2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let def   = Map2Data {stream1,stream2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.into());
        this
    }
}

impl<T1,T2,F,Out> EventConsumer<Output<T1>> for Map2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    fn on_event(&self, value1:&Output<T1>) {
        let value2 = self.stream2.value();
        let out    = (self.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<T1,T2,F> Debug for Map2Data<T1,T2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map2Data")
    }
}

impl<T1,T2,F> InputBehaviors for Map2Data<T1,T2,F>
    where T1:StreamOutput, T2:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.stream2)]
    }
}



// ============
// === Map3 ===
// ============

docs_for_map! {
pub struct Map3Data <T1,T2,T3,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, function:F }}
pub type   Map3     <T1,T2,T3,F> = StreamNode     <Map3Data<T1,T2,T3,F>>;
pub type   WeakMap3 <T1,T2,T3,F> = WeakStreamNode <Map3Data<T1,T2,T3,F>>;

impl<T1,T2,T3,F,Out> HasOutput for Map3Data<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,F,Out> Map3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let def   = Map3Data {stream1,stream2,stream3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,F,Out> EventConsumer<Output<T1>> for Map3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    fn on_event(&self, value1:&Output<T1>) {
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let out    = (self.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<T1,T2,T3,F> Debug for Map3Data<T1,T2,T3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map3Data")
    }
}

impl<T1,T2,T3,F> InputBehaviors for Map3Data<T1,T2,T3,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.stream2), Link::behavior(&self.stream3)]
    }
}



// ============
// === Map4 ===
// ============

docs_for_map! {
pub struct Map4Data <T1,T2,T3,T4,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, stream4:Watched<T4>, function:F }}
pub type   Map4     <T1,T2,T3,T4,F> = StreamNode     <Map4Data<T1,T2,T3,T4,F>>;
pub type   WeakMap4 <T1,T2,T3,T4,F> = WeakStreamNode <Map4Data<T1,T2,T3,T4,F>>;

impl<T1,T2,T3,T4,F,Out> HasOutput for Map4Data<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,T4,F,Out> Map4<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let stream4 = watch_stream(t4);
        let def   = Map4Data {stream1,stream2,stream3,stream4,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,T4,F,Out> EventConsumer<Output<T1>> for Map4<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    fn on_event(&self, value1:&Output<T1>) {
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let value4 = self.stream4.value();
        let out    = (self.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<T1,T2,T3,T4,F> InputBehaviors for Map4Data<T1,T2,T3,T4,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.stream2), Link::behavior(&self.stream3), Link::behavior(&self.stream4)]
    }
}

impl<T1,T2,T3,T4,F> Debug for Map4Data<T1,T2,T3,T4,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map4Data")
    }
}



// ==============
// === Apply2 ===
// ==============

macro_rules! docs_for_apply { ($($tt:tt)*) => { #[doc="
On every input event sample all input streams and run the provided function on all gathered values.
If you want to run the function only on event on the first input, use the `map` function family
instead.
"]$($tt)* }}

docs_for_apply! {
pub struct Apply2Data <T1,T2,F> { stream1:Watched<T1>, stream2:Watched<T2>, function:F }}
pub type   Apply2     <T1,T2,F> = StreamNode     <Apply2Data<T1,T2,F>>;
pub type   WeakApply2 <T1,T2,F> = WeakStreamNode <Apply2Data<T1,T2,F>>;

impl<T1,T2,F,Out> HasOutput for Apply2Data<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    type Output = Out;
}

impl<T1,T2,F,Out> Apply2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let def   = Apply2Data {stream1,stream2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.into());
        this
    }
}

impl<T1,T2,F,Out,T> EventConsumer<T> for Apply2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let out    = (self.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<T1,T2,F> Debug for Apply2Data<T1,T2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply2Data")
    }
}



// ==============
// === Apply3 ===
// ==============

docs_for_apply! {
pub struct Apply3Data <T1,T2,T3,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, function:F }}
pub type   Apply3     <T1,T2,T3,F> = StreamNode     <Apply3Data<T1,T2,T3,F>>;
pub type   WeakApply3 <T1,T2,T3,F> = WeakStreamNode <Apply3Data<T1,T2,T3,F>>;

impl<T1,T2,T3,F,Out> HasOutput for Apply3Data<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,F,Out> Apply3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let def   = Apply3Data {stream1,stream2,stream3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,F,Out,T> EventConsumer<T> for Apply3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let out    = (self.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<T1,T2,T3,F> Debug for Apply3Data<T1,T2,T3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply3Data")
    }
}



// ==============
// === Apply4 ===
// ==============

docs_for_apply! {
pub struct Apply4Data <T1,T2,T3,T4,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, stream4:Watched<T4>, function:F }}
pub type   Apply4     <T1,T2,T3,T4,F> = StreamNode     <Apply4Data<T1,T2,T3,T4,F>>;
pub type   WeakApply4 <T1,T2,T3,T4,F> = WeakStreamNode <Apply4Data<T1,T2,T3,T4,F>>;

impl<T1,T2,T3,T4,F,Out> HasOutput for Apply4Data<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,T4,F,Out> Apply4<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let stream4 = watch_stream(t4);
        let def     = Apply4Data {stream1,stream2,stream3,stream4,function};
        let this    = Self::construct(label,def);
        let weak    = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.clone_ref().into());
        t4.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,T4,F,Out,T> EventConsumer<T> for Apply4<T1,T2,T3,T4,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let value4 = self.stream4.value();
        let out    = (self.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<T1,T2,T3,T4,F> Debug for Apply4Data<T1,T2,T3,T4,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply4Data")
    }
}



// ========================
// === Network Node API ===
// ========================

impl Network {
    docs_for_source! {
    pub fn source<T:Data>(&self, label:Label) -> Stream<T> {
        self.register(Source::new(label))
    }}

    docs_for_source! {
    pub fn source_(&self, label:Label) -> Stream<()> {
        self.register(Source::new(label))
    }}

    docs_for_trace! {
    pub fn trace<M,S,T>(&self, label:Label, message:M, stream:&S) -> Stream<T>
    where M:Into<String>, S:StreamOutput<Output=T>, T:Data {
        self.register(Trace::new(label,message,stream))
    }}

    docs_for_toggle! {
    pub fn toggle<S:StreamOutput>(&self, label:Label, stream:&S) -> Stream<bool> {
        self.register(Toggle::new(label,stream))
    }}

    docs_for_count! {
    pub fn count<S:StreamOutput>(&self, label:Label, stream:&S) -> Stream<usize> {
        self.register(Count::new(label,stream))
    }}

    docs_for_constant! {
    pub fn constant<S,T> (&self, label:Label, stream:&S, value:T) -> Stream<T>
    where S:StreamOutput, T:Data {
        self.register(Constant::new(label,stream,value))
    }}

    docs_for_previous! {
    pub fn previous<S,T> (&self, label:Label, stream:&S) -> Stream<T>
    where S:StreamOutput<Output=T>, T:Data {
        self.register(Previous::new(label,stream))
    }}

    docs_for_sample! {
    pub fn sample<E:StreamOutput,B:StreamOutput>
    (&self, label:Label, event:&E, behavior:&B) -> Stream<Output<B>> {
        self.register(Sample::new(label,event,behavior))
    }}

    docs_for_gate! {
    pub fn gate<T,E,B>(&self, label:Label, event:&E, behavior:&B) -> Stream<Output<E>>
    where T:Data, E:StreamOutput<Output=T>, B:StreamOutput<Output=bool> {
        self.register(Gate::new(label,event,behavior))
    }}


    // === Merge ===

    docs_for_merge! {
    /// Please note that this function does output a more specific type than just `Stream<T>`. It is
    /// left on purpose so you could use the `add` method to build recursive data-stream networks.
    pub fn merge_<T:Data>(&self, label:Label) -> WeakMerge<T> {
        self.register_raw(Merge::new(label))
    }}

    docs_for_merge! {
    pub fn merge<T1,T2,T:Data>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T> {
        self.register(Merge::new2(label,t1,t2))
    }}

    docs_for_merge! {
    pub fn merge1<T1,T:Data>(&self, label:Label, t1:&T1) -> Stream<T>
    where T1:StreamOutput<Output=T> {
        self.register(Merge::new1(label,t1))
    }}

    docs_for_merge! {
    pub fn merge2<T1,T2,T:Data>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T> {
        self.register(Merge::new2(label,t1,t2))
    }}

    docs_for_merge! {
    pub fn merge3<T1,T2,T3,T:Data>(&self, label:Label, t1:&T1, t2:&T2, t3:&T3) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T>, T3:StreamOutput<Output=T> {
        self.register(Merge::new3(label,t1,t2,t3))
    }}

    docs_for_merge! {
    pub fn merge4<T1,T2,T3,T4,T:Data>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T>, T3:StreamOutput<Output=T>, T4:StreamOutput<Output=T> {
        self.register(Merge::new4(label,t1,t2,t3,t4))
    }}


    // === Zip ===

    docs_for_zip2! {
    pub fn zip<T1,T2>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<(Output<T1>,Output<T2>)>
    where T1:StreamOutput, T2:StreamOutput {
        self.register(Zip2::new(label,t1,t2))
    }}

    docs_for_zip2! {
    pub fn zip2<T1,T2>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<(Output<T1>,Output<T2>)>
    where T1:StreamOutput, T2:StreamOutput {
        self.register(Zip2::new(label,t1,t2))
    }}

    docs_for_zip3! {
    pub fn zip3<T1,T2,T3>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3) -> Stream<(Output<T1>,Output<T2>,Output<T3>)>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
        self.register(Zip3::new(label,t1,t2,t3))
    }}

    docs_for_zip4! {
    pub fn zip4<T1,T2,T3,T4>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4)
    -> Stream<(Output<T1>,Output<T2>,Output<T3>,Output<T4>)>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
        self.register(Zip4::new(label,t1,t2,t3,t4))
    }}


    // === Map ===

    docs_for_map! {
    pub fn map<S,F,T>(&self, label:Label, source:&S, f:F) -> Stream<T>
    where S:StreamOutput, T:Data, F:'static+Fn(&Output<S>)->T {
        self.register(Map::new(label,source,f))
    }}

    docs_for_map! {
    pub fn map2<T1,T2,F,T>(&self, label:Label, t1:&T1, t2:&T2, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->T {
        self.register(Map2::new(label,t1,t2,f))
    }}

    docs_for_map! {
    pub fn map3<T1,T2,T3,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->T {
        self.register(Map3::new(label,t1,t2,t3,f))
    }}

    docs_for_map! {
    pub fn map4<T1,T2,T3,T4,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->T {
        self.register(Map4::new(label,t1,t2,t3,t4,f))
    }}


    // === Apply ===

    docs_for_apply! {
    pub fn apply2<T1,T2,F,T>(&self, label:Label, t1:&T1, t2:&T2, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->T {
        self.register(Apply2::new(label,t1,t2,f))
    }}

    docs_for_apply! {
    pub fn apply3<T1,T2,T3,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->T {
        self.register(Apply3::new(label,t1,t2,t3,f))
    }}

    docs_for_apply! {
    pub fn apply4<T1,T2,T3,T4,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->T {
        self.register(Apply4::new(label,t1,t2,t3,t4,f))
    }}
}



///////////////////////////////////
//
//
//
//#[allow(unused_variables)]
//pub fn test() {
//    println!("hello");
//
////    new_network! { network
////        def source  = source::<f32>();
////        def source2 = source::<()>();
////        def tg      = toggle(&source);
////        def fff     = map(&tg,|t| { println!("{:?}",t) });
////        def bb      = sample(&source2,&tg);
////
////        let bb2 : Stream<bool> = bb.into();
////
////        def fff2   = map(&bb2,|t| { println!(">> {:?}",t) });
////        def m      = merge_::<usize>();
////        def c      = count(&m);
////        def t      = trace("t",&c);
////    }
////
////    m.add(&c);
////
////    println!("{:?}",tg);
////
////    source.emit(&5.0);
////    source2.emit(&());
////    source.emit(&5.0);
////    source2.emit(&());
////    source.emit(&5.0);
////
////    m.emit(&0);
////    m.emit(&0);
////    m.emit(&0);
//
////    network.draw();
//
//    crate::new_network! { network1
//        def source = source();
//        def count  = source.count();
//        def t      = trace("source",&source);
//        def t2     = trace("count",&count);
//    }
//
//    source.emit(());
//    source.emit(());
//    source.emit(());
//
//}



