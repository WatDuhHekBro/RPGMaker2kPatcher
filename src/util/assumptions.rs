// The purpose of this file is purely to clarify assumptions/shortcuts of the stdlib this project uses.
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    // Purpose: HashMap<(event, page), ...> ==> Offsets Table
    #[test]
    fn assumption_hashmap_should_correctly_use_optionals() {
        let mut blah: HashMap<(i32, Option<i32>), i32> = HashMap::new();
        blah.insert((7, None), 123);
        blah.insert((123, Some(456)), 69);
        blah.insert((1, Some(2)), 420);

        println!("{blah:?}");

        assert_eq!(blah.get(&(7, None)), Some(&123));
        assert_eq!(blah.get(&(123, Some(456))), Some(&69));
        assert_eq!(blah.get(&(1, Some(2))), Some(&420));
        assert_eq!(blah.get(&(6, Some(7))), None);
        assert_eq!(blah.get(&(6, None)), None);
        assert_eq!(blah.get(&(6, Some(6))), None);
    }

    // Usage: last_page: Option<i32> ==> if last_page == *page
    #[test]
    fn assumption_optionals_should_compare_inner_values() {
        let a: Option<i32> = None;
        let b: Option<i32> = Some(4);
        let c: Option<i32> = Some(4);
        let d: Option<i32> = Some(5);
        let e: Option<i32> = None;

        assert!(a == a);
        assert!(a != b);
        assert!(a != c);
        assert!(a != d);
        assert!(a == e);

        assert!(b == b);
        assert!(b == c);
        assert!(b != d);
        assert!(b != e);
    }

    // Usage: Test sorting (event, page) keys by event, then page.
    #[test]
    fn assumption_event_page_key_pair_should_be_sorted_correctly() {
        let mut keys: Vec<&(i32, Option<i32>)> = Vec::new();

        keys.push(&(69, Some(2)));
        keys.push(&(7, Some(3)));
        keys.push(&(420, None));
        keys.push(&(69, Some(4)));
        keys.push(&(5, Some(3)));
        keys.push(&(69, Some(1)));
        keys.push(&(420, Some(2)));
        keys.push(&(69, Some(3)));

        keys.sort_by(|a, b| a.cmp(b));

        assert_eq!(keys[0], &(5, Some(3)));
        assert_eq!(keys[1], &(7, Some(3)));

        assert_eq!(keys[2], &(69, Some(1)));
        assert_eq!(keys[3], &(69, Some(2)));
        assert_eq!(keys[4], &(69, Some(3)));
        assert_eq!(keys[5], &(69, Some(4)));

        assert_eq!(keys[6], &(420, None));
        assert_eq!(keys[7], &(420, Some(2)));
    }

    #[test]
    fn assumption_toml_value_serialize_7f() {
        let mut value = String::new();

        serde::Serialize::serialize(
            &String::from("as\\\\d\x7Ff\nasdf\nz"),
            toml::ser::ValueSerializer::new(&mut value),
        )
        .unwrap();

        assert_eq!(
            value,
            r#""""
as\\\\d\u007Ff
asdf
z""""#
        );
    }
}
