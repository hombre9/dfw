
error_chain! {
    links {
        Another(::boondock::errors::Error, ::boondock::errors::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        TomlSer(::toml::ser::Error);
        TomlDe(::toml::de::Error);
    }
}
