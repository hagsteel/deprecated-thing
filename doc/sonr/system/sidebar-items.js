initSidebarItems({"enum":[["SystemEvent","Specific event the [`System`] responds to NOTE: There should only be one [`System::init`] call per thread The System handles registration and pushes `Event`s to the reactors passed to [`System::start`]."]],"struct":[["System","`System` is thread local and has to exist for each thread using `Reactors` that react to [`Reaction::Event(event)`].  There can only be one instance of a `System` per thread, and this instance is created by  calling `System::init()`."]]});