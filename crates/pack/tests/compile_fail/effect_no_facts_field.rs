// Prove: AgentEffect has no `facts` field — direct fact emission is impossible.
// This file must FAIL to compile.

use converge_pack::AgentEffect;

fn main() {
    let effect = AgentEffect::empty();
    let _facts = effect.facts;
}
