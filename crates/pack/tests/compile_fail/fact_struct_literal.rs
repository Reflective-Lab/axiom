// Prove: Fact fields are private — struct literal construction is impossible.
// This file must FAIL to compile.

use converge_pack::{ContextKey, Fact};

fn main() {
    let _fact = Fact {
        key: ContextKey::Seeds,
        id: "test-id".to_string(),
        content: "test-content".to_string(),
    };
}
