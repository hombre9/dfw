
error_chain! {
    foreign_links {
        Docker(::shiplift::errors::Error);
        IPTError(::iptables::error::IPTError);
        Io(::std::io::Error);
        ParseError(::url::ParseError);
        ParseIntError(::std::num::ParseIntError);
        TomlSer(::toml::ser::Error);
        TomlDe(::toml::de::Error);
    }
}
