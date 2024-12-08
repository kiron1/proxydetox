use boa_engine::{class::Class, object::builtins::JsArray, JsError, JsNativeError};
use boa_gc::{Finalize, Trace};

type Map = std::collections::HashMap<String, Entry>;

#[derive(Debug, Default, Trace, Finalize)]
pub(crate) struct Table {
    root: Map,
}

#[derive(Debug, Default, Trace, Finalize)]
struct Entry {
    root: Map,
}

impl Table {
    pub(crate) fn contains<A: AsRef<str>>(&self, domain: A) -> bool {
        let domain = Domain(domain.as_ref());
        let parts = domain.parts();
        let mut root = &self.root;
        for p in parts {
            if let Some(next) = root.get(p) {
                root = &next.root;
                if next.is_leaf() {
                    return true;
                }
            } else {
                break;
            }
        }
        false
    }
}

impl Entry {
    fn is_leaf(&self) -> bool {
        self.root.is_empty()
    }
}

impl<A> FromIterator<A> for Table
where
    A: AsRef<str>,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut result = Self {
            root: Default::default(),
        };
        for domain in iter {
            let domain = Domain(domain.as_ref());
            let parts = domain.parts();
            let mut root = &mut result.root;
            for p in parts {
                root = &mut root.entry(p.to_owned()).or_default().root;
            }
        }
        result
    }
}

struct Domain<'a>(&'a str);

impl Domain<'_> {
    fn parts(&self) -> impl Iterator<Item = &str> {
        self.0
            .trim_matches(|c: char| c.is_ascii_whitespace() || c == '.')
            .split('.')
            .rev()
    }
}

impl Class for Table {
    const NAME: &'static str = "DomainTable";

    fn constructor(
        _this: &boa_engine::JsValue,
        args: &[boa_engine::JsValue],
        context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        let Some(domains) = args.first() else {
            return Err(JsNativeError::typ()
                .with_message("first argument is missing")
                .into());
        };
        let domains = domains
            .as_object()
            .ok_or_else::<JsError, _>(|| {
                JsNativeError::typ()
                    .with_message("first argument must be an array")
                    .into()
            })
            .and_then(|o| JsArray::from_object(o.clone()))?;

        // TODO: avoid the temporary vector by using an Iterator directly, however it looks like,
        // that JsArray does not expose an iterator interface in the Rust world.
        let len = domains.length(context)?;
        let mut domains_vec = Vec::<String>::with_capacity(len as usize);
        for k in 0..len {
            let domain = domains.at(k as i64, context)?;
            let domain = domain
                .as_string()
                .ok_or_else::<JsError, _>(|| {
                    JsNativeError::typ()
                        .with_message("domain {k} is not a string")
                        .into()
                })
                .map(|s| s.to_std_string_escaped())?;
            domains_vec.push(domain);
        }
        let table = domains_vec.into_iter().collect::<Table>();
        Ok(table)
    }

    fn init(class: &mut boa_engine::class::ClassBuilder) -> boa_engine::JsResult<()> {
        class.method(
            "contains",
            1,
            boa_engine::NativeFunction::from_fn_ptr(|this, args, _ctx| {
                let Some(object) = this.as_object() else {
                    return Err(JsNativeError::typ()
                        .with_message("'this' is not an object'")
                        .into());
                };
                let Some(table) = object.downcast_ref::<Table>() else {
                    return Err(JsNativeError::typ()
                        .with_message("'this' is not a DomainTable'")
                        .into());
                };
                let Some(domain) = args.first() else {
                    return Err(JsNativeError::typ()
                        .with_message("first argument is missing")
                        .into());
                };
                let Some(domain) = domain.as_string() else {
                    return Err(JsNativeError::typ()
                        .with_message("first argument must be a string")
                        .into());
                };
                let domain = domain.to_std_string_escaped();
                Ok(table.contains(domain).into())
            }),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_domain_table() {
        let table = ["example.org", "example.net", "test.org"]
            .iter()
            .collect::<Table>();
        assert_eq!(table.root.len(), 2);
        assert!(table.contains("example.org"));
        assert!(table.contains("www.example.org"));
        assert!(!table.contains("example.info"));
        assert!(!table.contains("net"));
        assert!(!table.contains("org"));
    }
}
