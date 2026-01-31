use easy_macros_add_code::add_code;

#[add_code(before = { events.push("setup"); })]
fn example_body_only(events: &mut Vec<&'static str>) {
    // Body stays minimal for docify examples.
    events.push("example");
}

#[add_code(after = { events.push("teardown"); })]
fn after_with_return(events: &mut Vec<&'static str>) -> usize {
    events.push("example");
    42
}

#[add_code(
    before = { events.push("setup"); },
    after = { events.push("teardown"); }
)]
fn setup_and_teardown(events: &mut Vec<&'static str>) {
    events.push("example");
}

#[test]
fn injects_setup_before_example() {
    let mut events = Vec::new();
    example_body_only(&mut events);
    assert_eq!(events, vec!["setup", "example"]);
}

#[test]
fn injects_teardown_after_example_and_preserves_return() {
    let mut events = Vec::new();
    let value = after_with_return(&mut events);
    assert_eq!(value, 42);
    assert_eq!(events, vec!["example", "teardown"]);
}

#[test]
fn injects_setup_and_teardown_in_order() {
    let mut events = Vec::new();
    setup_and_teardown(&mut events);
    assert_eq!(events, vec!["setup", "example", "teardown"]);
}

#[add_code(after = { Ok(()) })]
#[test]
fn injects_ok_end_for_docify_style() -> Result<(), ()> {
    let mut events = Vec::new();
    example_body_only(&mut events);
    assert_eq!(events, vec!["setup", "example"]);
}
