#[macro_use]
extern crate macro_state;

write_state!("top_of_file", "value 1");

#[cfg(test)]
mod tests {
    write_state!("top of module", "value 2");

    #[test]
    fn test_write_state() {
        write_state!("top of method", "value 3");
        assert_eq!(read_state!("top_of_file"), "value 1");
        assert_eq!(read_state!("top of module"), "value 2");
        assert_eq!(read_state!("top of method"), "value 3");
    }

    #[test]
    fn test_rewriting_state() {
        write_state!("key 1", "value 4");
        assert_eq!(read_state!("key 1"), "value 4");
        write_state!("key 1", "value 5");
        assert_eq!(read_state!("key 1"), "value 5");
    }

    #[test]
    fn test_has_state() {
        assert_eq!(has_state!("key A"), false);
        write_state!("key A", "value 6");
        assert_eq!(has_state!("key A"), true);
        assert_eq!(read_state!("key A"), "value 6");
    }

    #[test]
    fn test_clear_state() {
        write_state!("key B", "value 7");
        assert_eq!(read_state!("key B"), "value 7");
        clear_state!("key B");
        assert_eq!(has_state!("key B"), false);
    }

    #[test]
    fn test_init_state() {
        write_state!("key C", "value 8");
        assert_eq!(init_state!("key C", "value -8"), "value 8");
        assert_eq!(init_state!("key D", "value 9"), "value 9");
        assert_eq!(init_state!("key C", "value -8"), "value 8");
        assert_eq!(init_state!("key D", "value 9"), "value 9");
    }
}
