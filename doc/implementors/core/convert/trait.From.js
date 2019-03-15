(function() {var implementors = {};
implementors["sonr"] = [{text:"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sonr/prelude/struct.Event.html\" title=\"struct sonr::prelude::Event\">Event</a>&gt; for <a class=\"enum\" href=\"sonr/reactor/enum.Reaction.html\" title=\"enum sonr::reactor::Reaction\">Reaction</a>&lt;T&gt;",synthetic:false,types:["sonr::reactor::Reaction"]},{text:"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> + <a class=\"trait\" href=\"sonr/trait.Evented.html\" title=\"trait sonr::Evented\">Evented</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sonr/reactor/struct.EventedReactor.html\" title=\"struct sonr::reactor::EventedReactor\">EventedReactor</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"sonr/net/stream/struct.Stream.html\" title=\"struct sonr::net::stream::Stream\">Stream</a>&lt;T&gt;",synthetic:false,types:["sonr::net::stream::Stream"]},{text:"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"sonr/sync/enum.Capacity.html\" title=\"enum sonr::sync::Capacity\">Capacity</a>&gt; for <a class=\"struct\" href=\"sonr/sync/signal/struct.SignalReceiver.html\" title=\"struct sonr::sync::signal::SignalReceiver\">SignalReceiver</a>&lt;T&gt;",synthetic:false,types:["sonr::sync::signal::SignalReceiver"]},{text:"impl&lt;T, '_&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;&amp;'_ <a class=\"enum\" href=\"sonr/sync/enum.Capacity.html\" title=\"enum sonr::sync::Capacity\">Capacity</a>&gt; for <a class=\"struct\" href=\"sonr/sync/signal/struct.SignalReceiver.html\" title=\"struct sonr::sync::signal::SignalReceiver\">SignalReceiver</a>&lt;T&gt;",synthetic:false,types:["sonr::sync::signal::SignalReceiver"]},{text:"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"sonr/sync/enum.Capacity.html\" title=\"enum sonr::sync::Capacity\">Capacity</a>&gt; for <a class=\"struct\" href=\"sonr/sync/broadcast/struct.Broadcast.html\" title=\"struct sonr::sync::broadcast::Broadcast\">Broadcast</a>&lt;T&gt;",synthetic:false,types:["sonr::sync::broadcast::Broadcast"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"sonr/errors/enum.Error.html\" title=\"enum sonr::errors::Error\">Error</a>",synthetic:false,types:["sonr::errors::Error"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;RecvError&gt; for <a class=\"enum\" href=\"sonr/errors/enum.Error.html\" title=\"enum sonr::errors::Error\">Error</a>",synthetic:false,types:["sonr::errors::Error"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;TryRecvError&gt; for <a class=\"enum\" href=\"sonr/errors/enum.Error.html\" title=\"enum sonr::errors::Error\">Error</a>",synthetic:false,types:["sonr::errors::Error"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/net/parser/struct.AddrParseError.html\" title=\"struct std::net::parser::AddrParseError\">AddrParseError</a>&gt; for <a class=\"enum\" href=\"sonr/errors/enum.Error.html\" title=\"enum sonr::errors::Error\">Error</a>",synthetic:false,types:["sonr::errors::Error"]},{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.FromUtf8Error.html\" title=\"struct alloc::string::FromUtf8Error\">FromUtf8Error</a>&gt; for <a class=\"enum\" href=\"sonr/errors/enum.Error.html\" title=\"enum sonr::errors::Error\">Error</a>",synthetic:false,types:["sonr::errors::Error"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
