# RFC Draft: Parameterized circuit construction and bounded compile-time evaluation (M2)

## Status and decision requested

**Draft / Proposed — English revision after live review, 2026-09-11. Not Accepted, not implemented.** Formal RFC number is unassigned. This English document is the working authority for the revision; the Chinese full translation is synchronized to this revision on 2026-09-11. If wording diverges, this English text governs. The implementation plan must be updated after the scope decision, not executed from its earlier checklist.

**Requested decision:** define bounded, typed compile-time computation and repetition that compose with the existing fn, array and subdesign contracts. Compare Candidate A (operation loops over declared objects, with ordinary effectful fn calls) and Candidate B (A plus direct iteration-local construction). **The earlier recommendation of Candidate B is withdrawn. Neither candidate is selected by this revision.** Shared arithmetic, parameter, checking and budget rules below apply to both; candidate-specific rules are marked explicitly.

The non-normative `docs/language-design-principles.md` supplies review questions, not a second language specification. Acceptance still requires a decision record and synchronized changes to note 10 and affected RFCs. Proposed syntax and prospective tests are not evidence of implementation.

## Binding baseline and division of responsibilities

Main was pulled on 2026-09-10 at `0e3d7705ab393a5c64c20836540ad1bc2e49898e`. [RFC-032](https://github.com/conol-ai/cohdl/blob/0e3d7705ab393a5c64c20836540ad1bc2e49898e/docs/design/rfc-032-subdesign.md) is Accepted (2026-09-08) and included in [note 10](https://github.com/conol-ai/cohdl/blob/0e3d7705ab393a5c64c20836540ad1bc2e49898e/docs/design/10-language-specification.md). Apply AGENTS.md's authority order and the [implementation ledger](https://github.com/conol-ai/cohdl/blob/0e3d7705ab393a5c64c20836540ad1bc2e49898e/docs/compliance-report.md). This branch's closed historical harness proposal uses the same number in a different file; it provides no language authority. The earlier exclusion of subdesign on that basis was an authoring error and is withdrawn.

| Construct | Responsibility retained or proposed |
| --- | --- |
| `module` | Existing names, imports and visibility through RFC-016 |
| `subdesign` | Existing Pin ports, named hierarchy, shared generic substitution, arrays, nested composition and default relative layout with whole-unit placement and internal placement overrides |
| Physical instance array | Existing named family of real components; M2 makes lengths and selectors computable |
| Subdesign array | Existing named family of logical composition nodes with real children; M2 uses the same count/index evaluator |
| `fn` | Existing effectful inline circuit fragment with Pin/Instance arguments; locals are real components but have no externally placeable interface |
| `for` | Proposed repetition of admitted statements; it introduces a lexical iteration scope, not a port interface or coordinate frame |
| Pure expression | Proposed Int/Length computation without circuit effects or I/O; a value-returning pure fn model is outside this amendment |

Electrical access across a subdesign boundary remains through ports. Placement alone may reach internal instances. A subdesign is not a physical Instance, has no part/designator/BOM row, and cannot satisfy an Instance-typed fn parameter. Physical pins retain final assembly obligations; required ports additionally obey the existing outside-the-node connectivity check. `nc` on a port remains E1306, and a subdesign use site inside a fn remains E1307, including through a loop. No candidate removes fn's existing ability to construct parts.

**Known discrepancies:** RFC-032/note 10 call fn same-package, while RFC-016 resolution and existing `pub fn` support cross-package calls. Retained interfaces and placement access are the sound distinction; the conflicting package wording needs governance correction. RFC-032 also requires hierarchy in checked IR, whereas current public `src/ir.rs` exposes a flat physical design and the expander retains subdesign nodes internally. Child paths alone do not prove a complete public hierarchy model or complete Explorer support. This RFC neither revokes RFC-032 nor treats implementation gaps as its permission to omit subdesign.

The remaining scope decisions are concrete: A versus B; whether all loops need an authored identity label; whether to add half-open fan-out alongside the existing inclusive selector. The design below specifies a labelled-loop comparison baseline and preserves current fan-out syntax, without claiming these spelling choices are Accepted. The full RC workflow and candidate comparison must precede selecting a construction scope.

## Problem

CoHDL already describes devices, physical instances, connections, reusable fn definitions and Accepted subdesign composition. RFC-024 adds `[Device; N]` with literal counts and indexes, but explicitly leaves count-dependent wiring and placement manual. A board family also needs to repeat a whole channel: instantiate its resistor and capacitor, connect both, and compose that channel into another reusable definition.

An RFC that only adds `for` leaves the central questions unanswered: what may a program compute, what may it construct, how do values differ from physical objects, and what checks survive generation? Users need to change a channel count or reuse a circuit without losing unit safety, pin obligations, stable physical identity or the compiler's ability to explain a failure.

The intended authors are AI generating/repairing `.cohdl`, humans reviewing the resulting design, and library authors composing parameterized circuits. The three acceptance scenarios below are language-contract tests, not claims that an LED chain or RC network is a validated hardware product.

## Goals

1. Define CoHDL programmability independently of any single loop spelling, with explicit inputs, value domains, construction effects and checking stages.
2. Express a daisy chain, a repeated multi-component channel, and nested parameterized fn/subdesign composition through shared mechanisms.
3. Preserve unit/trait/parameter-kind checking before construction and every real instance's connection obligations after assembly.
4. Reject concrete value errors and excessive expansion deterministically, retaining the source expression and distinguishing call/iteration context.
5. Preserve existing array identity and legacy outputs, and specify the edit-stability limits of newly generated objects.
6. Deliver one semantics for check/build, fmt, LSP, docs and Explorer; cohdl.ai consumes it rather than reimplementing it.

These serve Constitution ranks 1–5: correctness/gradeability, AI-generatability, human reviewability, composability and faithful output. Reduced typing is a benefit; it does not establish correctness.

## Capability model: what CoHDL should be able to program

**CoHDL programmability means using explicit parameters and checked compile-time computation to construct a finite electrical design whose instances, connections and declared constraints remain subject to the ordinary compiler contracts.** It is design-time computation; no loop or variable becomes firmware executing on the manufactured board.

The full capability model and the scope proposed for M2 are deliberately distinct:

| Capability | Meaning for CoHDL | This M2 proposal / remaining work |
| --- | --- | --- |
| Parameter and value computation | Derive design data from explicit inputs | Add Int/Length expressions, local constants and fn/subdesign count parameters; retain other existing unit-literal/parameter forwarding. General dimensional formulas and value-returning functions require a later contract |
| Circuit construction | Create real instances and explicit connections from those data | Both candidates use ordinary declarations and explicit net/nc/fn operations; B additionally admits direct scalar/array inst and legal subdesign use sites inside loops; no source-string generation |
| Repetition and composition | Build N channels and reuse them inside another definition | Generalize both physical and subdesign arrays; compose finite for with existing fn/subdesign and shared generic binding. Fn remains effectful; loop labels are not returned circuit objects |
| Configuration | Choose a circuit form or fixed device from declared alternatives | Preserve existing type arguments and RFC-008 structural variants; zero-or-more helper invocation is already count-controlled; B also controls direct local declarations; general Bool/if, branch selection and branch-specific checking are deferred |
| Domain recipes | Given voltage/current requirements, derive component parameters and select suitable parts | M3 must specify dimensional formulas, applicable device facts, selection policy and pinned results; M2 provides composition/count mechanisms, not that solver |
| Checkable electrical contracts | State and verify obligations on the chosen/generated design | Preserve current unit/trait/pin checks and exactly four residual DRC rules. M4 must classify additional contracts, evidence and missing-data behavior; it is not a promise of arbitrary runtime assert |

An ordinary parameterized fn can already forward explicit resistance/capacitance values. Deferring general electrical formulas does not defer those existing typed values. Similarly, a fixed part passed explicitly is not automatic parts selection. An MVP can implement a bounded slice of this model without calling that slice a general-purpose programming language.

### Values, references and construction effects

Three categories must remain distinct:

- **Values:** Int counts/indexes and physical quantities. Evaluation produces a value and performs no circuit mutation, I/O, registry query or allocation of a physical component. M2 expands arithmetic only for Int and Length.
- **References:** physical Instance/Pin bindings, legal logical-port references and indexed array elements. A named subdesign node has a different kind from a physical Instance; neither a bare array nor a subdesign node can impersonate an Instance or Pin. They cannot be added, serialized into generated identifiers or inspected for undeclared electrical facts.
- **Construction effects:** inst contributes a physical instance; net contributes connectivity; nc contributes an explicit non-connection decision; fn contributes its expanded fragment; a subdesign use contributes a logical node and its real contents; layout contributes existing placement/constraint facts. For repeats admitted construction effects. A const creates none of them.

A scalar inst declaration creates one real instance per entered frame; an array inst declaration creates its evaluated positive length of real elements per entered frame. Referencing that instance twice never constructs two components. Calling an effectful fn twice does construct two copies of its locals; it cannot be memoized away as if it were a pure expression. There is no object-return/collection API, mutable graph API or hidden auto-wiring in M2. Named physical/subdesign arrays provide externally addressable families. Candidate B adds private iteration-local declarations; their reduced external addressability is a cost to compare, not proof that this category is needed.

### The checking contract is part of the capability

Typed input does not by itself prove that expansion yields a valid circuit. This RFC requires a staged contract:

| Stage | Information available | Required obligation |
| --- | --- | --- |
| Definition/type validation | Declared types, names, traits and concrete subexpressions | Resolve symbols and pin members; check legal parameter kinds, unit compatibility and new constructs even in unused/empty bodies, within the compatibility boundary in Design §8 |
| Binding and finite elaboration | Actual generic values, array lengths and each entered loop value | Check integer domains, bounds, precision, operation context and budgets; create hygienic objects with provenance |
| Final design assembly | All generated instances, nets and nc decisions | Apply physical-pin obligations and RFC-032 required-port checks after fn/subdesign/loop expansion, preserving existing connection merging/conflict checks |
| Residual DRC | The assembled graph and explicitly modeled electrical facts | Run the existing four rules unchanged; structural index/type errors never become a fifth rule |
| Build and projection | Checked design plus parts, lock and referenced resources | Enforce existing part/footprint/allocation/output contracts for every real instance; no fake loop BOM rows or partial successful build |

This is **type-system-first, not type-system-only**. An Int can still be out of bounds, a well-typed connection can join the wrong intended channel, and passing existing DRC does not prove analog response. Those require concrete value checking, an independent topology oracle, or domain/physical evidence respectively. A Pin parameter is not a newly introduced voltage-indexed dependent type. We do not prove every possible const-generic instantiation valid: each actual activation is checked, with invariant errors rejected earlier when sufficient information exists.

## Relationship to the CoHDL type system

### Extend typed construction, preserve the existing electrical model

M2 extends the language's compile-time value and construction mechanisms. It does not replace Device, Part, Trait, Pin or their existing contracts. The distinction matters: adding arithmetic to a count does not create a new electrical unit, and repeating a declaration does not weaken the type of the declared device.

| Existing mechanism | M2 relationship | Concrete acceptance obligation |
| --- | --- | --- |
| RFC-001 physical unit types | Preserve all existing unit identities and zero coercion; add the specified exact Length arithmetic, with Int as a separate structural value domain | `100nF` cannot bind R: Resistance; `10mm + 2` fails, while `10mm + 2 * 1mm` has type Length |
| RFC-003 trait satisfaction | Every concrete device substitution still needs satisfying impls and valid trait pin/spec maps | Passing an otherwise valid device that lacks the required Capacitor impl fails even when reached through a repeated helper |
| RFC-007 generic parameters | Add structural `const N: Int` to fn and subdesign through the same binding/substitution mechanism; retain unit-value and trait-bound type parameters | Forwarding N, R and C through nested calls preserves their distinct kinds; defaults do not erase the expected kind |
| RFC-024/032 arrays | Share count/index expressions while preserving physical-versus-logical element kind | Positive Int lengths for both; selected physical pins and subdesign ports retain their own obligations |
| RFC-006 fn and reference binding | Reuse Pin/Instance binding and effectful fragment expansion | A Pin argument refers to an existing physical pin or a permitted subdesign port; calling a helper twice creates two sets of its locals, not two copies of the passed-in object |
| RFC-002 pin obligations | Preserve exhaustiveness at final assembly | Every actual physical instance and required subdesign port participates, including one whose missing pin is only exposed after caller wiring is complete |
| RFC-004 residual DRC | Preserve the four existing graph-level rules | Neither a type mismatch nor a concrete index error is routed to a new DRC rule |

### Three kinds of generic input, one substitution discipline

In a declaration such as `fn bank<const N: Int, R: Resistance, D: Capacitor>(target: D, pin: Pin)`, N is a structural integer value, R is a Resistance value, and D is a device type constrained by a trait. They are not interchangeable spellings for an untyped parameter. `target` is an instance reference checked against D, and `pin` is a pin reference; neither is a computed scalar.

The current implementation expresses the existing split through `GenericBound::Unit`/`Traits` and `GenericValue::Unit`/`Device` (`src/ast.rs`, `src/check/generics.rs`). M2 needs a distinct Int case and a default/argument representation capable of preserving its type and span. It must not encode N as a fake UnitValue, infer its kind from a string, or add a second generic resolver only for loops. These implementation seams explain the relationship; the exact Rust variant names are not public language syntax.

Similarly, existing trait bounds describe the obligations of a device, not arbitrary predicates over count values. N > 0 is checked at an array-length use; it is not silently introduced as a new trait. General where-clauses, refinement types and a theorem prover over arbitrary integers are outside M2.

### A known type is different from a known value

Before binding a generic N, the compiler can know that N is Int without knowing whether N is 3, 0 or negative. It can therefore reject `N + 1mm` during type validation. Whether `[D; N]` has a valid positive length is checked when N becomes concrete. Likewise, `leds[n + 1]` can have a correctly typed index while still failing the concrete bound check on the last iteration.

Array shape is value-dependent in this limited sense; M2 does not introduce a general dependent type system, compile-time proofs of all possible indexes, or runtime bounds checks on the physical board. It checks each actual instantiation/iteration and rejects invalid construction before accepting the design. The empty-loop rules in Design §8 still reject known invariant failures without inventing values for unbound loop variables.

```cohdl
// Proposed negative example: no concrete value for C is needed to find the unit error.
fn wrong_channel<C: Capacitance>() {
    inst r: SeriesR<C>
}
```

The uncalled definition must fail because SeriesR expects Resistance and C has type Capacitance. Candidate B must reject the same mismatch inside an empty loop. Neither test needs a concrete value for C. By contrast, a division depending on the unbound iteration value has the precise deferred-check boundary specified in Design §8. This is a distinction between type information and value information, not permission to skip checking an unused body.

### Connection obligations compose after construction

An ordinary `Pin` type establishes reference kind; it does not assert that the pin has a particular runtime voltage or maximum safe current. Those facts must come from existing modeled specs/annotations and applicable checks, or from a future electrical-contract proposal. The type of a resistor parameter therefore cannot prove that the entire RC network has the intended transfer function.

A helper can connect only part of a caller-owned device. Its unresolved required pins remain obligations of the final design; they must not be either rejected prematurely or discarded when the helper returns. For repeats this same fragment behavior. After all real instances and connections are known, the existing final assembly checks every obligation, preserving the distinction between an unmentioned required pin, an explicit nc and a connected pin.

The construction-preservation requirement is: under the declared source correspondence, each generated instance has the same resolved device/specs, trait-role mappings and pin obligations as its explicit counterpart; the resulting graph passes the same ordinary checks. This is an implementation acceptance property verified with positive and negative fixtures, not a claim that this RFC contains a formal soundness proof. Connectivity can still be wrong for the user's intent while all those checks pass; the independent topology oracle remains necessary.

### The type boundary determines implementation order

Implement typed expressions and shared generic substitution before accepting loop-generated objects. Then add declaration validation for new frames, concrete evaluation/expansion, and final-graph/provenance integration. A parser-only for, a loop-specific string substitution pass, or an editor that expands arrays independently cannot satisfy the contract. Tooling must report the same distinction between a symbolic type, a bound value and an assembled circuit that the compiler uses.

## Three scenarios and the complete layout workflow

The following snippets use proposed syntax and synthetic device interfaces. They are design comparisons, not executed M2 programs. Existing hand-written fixtures retain their dated evidence. Real part-bound tests are separate from synthetic topology checks and do not establish analog response or manufacturability.

### LED chain: compute connections over a physical array

```cohdl
// Proposed fragment; host and AddressableLED interfaces are supplied by the fixture.
const N: Int = 10
inst leds: [AddressableLED; N]
net VCC [5V]: host.V5, leds[0..=(leds.len - 1)].VDD
net GND [gnd]: host.GND, leds[0..=(leds.len - 1)].GND
net DATA: host.DATA, leds[0].DIN
for links: n in 0..(leds.len - 1) {
    net _: leds[n].DOUT, leds[n + 1].DIN
}
nc: leds[leds.len - 1].DOUT
```

Both candidates express this without direct inst in a loop. N = 1/2/10 contributes 0/1/9 neighbor nets. Power fan-out creates one shared net; it has inclusive endpoints because that is the inherited selector syntax, not because one-net semantics require an inclusive range. The loop is half-open. Design §4 records the spelling cost and the alternative to compare.

The draft label `links` identifies generated operations/provenance, not the LED array, a placement group or a subdesign. The LEDs keep their existing array identities. An out-of-range `n + 1` fails even though it is Int; removing power connections still exposes required physical pins. An explicit power-pin nc satisfies the existing structural decision rule but does not prove powered operation is appropriate.

### RC channels: compare the entire board-authoring task

First express the existing reusable unit. Only the later const, computed lengths/indexes and loops are M2 additions:

```cohdl
subdesign RcChannel<R: Resistance, C: Capacitance> {
    ports {
        required IN: Pin
        required OUT: Pin
        required GND: Pin
    }
    inst r: SeriesR<R>
    inst c: ShuntC<C>
    net _: IN, r.A
    net _: OUT, r.B, c.A
    net _: GND, c.B
    layout {
        place r at (0mm, 0mm)
        place c at (3mm, 0mm)
    }
}

design FilterBoard {
    const N: Int = 10
    inst inputs: [SignalSource; N]
    inst outputs: [SignalSink; N]
    inst ground: Ground
    subdesign channels: [RcChannel<1kohm, 100nF>; N]

    for wiring: i in 0..channels.len {
        net _: inputs[i].OUT, channels[i].IN
        net _: channels[i].OUT, outputs[i].IN
        net GND [gnd]: ground.GND, channels[i].GND
    }

    layout {
        for placement: i in 0..channels.len {
            place channels[i] at (10mm + i * 8mm, 15mm)
            place inputs[i] at (10mm + i * 8mm, 5mm)
            place outputs[i] at (10mm + i * 8mm, 25mm)
        }
        place ground at (0mm, 15mm)
        place channels[6].c at (62mm, 17mm)
    }
}
```

Both candidates can use this form. It has 2N passives, 2N signal interfaces and one ground interface: 4N+1 physical instances and 2N+1 electrical net classes. Container and phantom port counts are not component/BOM counts. Ground joins through explicitly shared connectivity; equal printed names are not a new cross-boundary connection rule. Incorrect R/C units, missing required ports, missing internal physical-pin connections and erroneous still-well-typed channel joins need distinct failure probes.

The review workflow is: **define → instantiate ten → wire ports → place separately → adjust capacitor seven → grow to twelve**. At N = 10, channel 6's default capacitor origin would be (61mm, 15mm); its explicit override is (62mm, 17mm). At N = 12, that override and all earlier channel origins must remain. Starting from the ten-channel lock, surviving `FilterBoard::channels_0::r` through `channels_9::c` keep paths, designators and their intended terminal relationships. Compare connectivity restricted to surviving endpoints: the shared ground class legitimately gains new endpoints. Also verify two new R/C pairs, new interface instances and complete part/BOM coverage. No loop segment may be inserted into an already-declared channel's path merely because a loop wires or places it.

Candidate B can alternatively put r/c declarations directly in a loop. That can express the electrical topology, but its private locals lack the separate external override interface above. Moving the placement inside the construction loop does not meet the independent-layout editing task. Switching B's source to the same subdesign form restores that task but demonstrates the shared baseline's capability, not the necessity of B's extra declaration surface. Retain direct-local examples as optional scope/type probes, not the flagship proof that CoHDL needs them.

### Nested reuse: shared substitution through logical boundaries and helpers

```cohdl
fn join(src: Pin, dst: Pin) {
    net _: src, dst
}

subdesign FilterBank<const N: Int, R: Resistance, C: Capacitance> {
    ports {
        required IN: Pin
        required GND: Pin
    }
    inst receivers: [SignalSink; N]
    subdesign channels: [RcChannel<R, C>; N]
    for wiring: i in 0..N {
        join(IN, channels[i].IN)
        join(channels[i].OUT, receivers[i].IN)
        net _: GND, channels[i].GND
    }
    layout {
        for placement: i in 0..N {
            place channels[i] at (i * 8mm, 0mm)
            place receivers[i] at (i * 8mm, 10mm)
        }
    }
}

// Proposed fragment in a design with declared source and ground interfaces:
subdesign bank: FilterBank<3, 1kohm, 100nF> {
    IN: source.OUT, GND: ground.GND,
}
layout {
    place bank at (10mm, 15mm)
    place bank.channels[1].c at (22mm, 17mm)
}
```

Both candidates pass N/R/C through shared generic substitution, use existing Pin binding in join, and preserve nested relative layout and board-level override paths. For N = 3 this shared-input scenario has 11 physical instances and five electrical net classes, including source and ground. It is not topology-equivalent to the independent-input RC workflow above. Compare each against its own explicit reference and compare A/B only under the same interfaces, component parameters and layout requirements.

Fn remains effectful: add a separate regression that repeats a current decoupling helper and proves one real capacitor per invocation, with valid physical-pin attribute targets and unchanged part obligations. A whole subdesign still cannot stand in for an Instance argument, and a logical port is not automatically a valid physics-attribute target. Making all fn pure would require a different migration proposal.

### Evidence needed before selecting A or B

| Dimension | Candidate A: operations + existing composition | Candidate B: add loop-local construction |
| --- | --- | --- |
| LED chain and index arithmetic | Expressed directly | Same expression; extra declarations unnecessary here |
| N-way RC with independent layout and individual overrides | Named subdesign array supplies the interface | Same subdesign form works; direct private locals alone do not supply it |
| Nested generic reuse | Existing subdesign/fn plus shared parameters | Same baseline works; additional per-iteration definitions remain to justify |
| Real objects created through fn calls | Allowed, retaining fn's current behavior | Allowed; not an exclusive benefit of direct inst |
| Different per-iteration local construction | May need a helper or a separately specified composition pattern | Direct declarations may help; requires an actual workload beyond assuming identical RC channels need it |
| Permanent cost | Loops, values, shared indexing, provenance and budgets | All of A plus local declaration visibility, generated-object identity and placement/access restrictions |

This is a source-level comparison, not an executed M2 result or a claim that A covers every future workload. A is the comparison baseline because it composes existing mechanisms. Select B only with evidence that its additional capability justifies its costs; select A only with honest disclosure of any remaining workload limitation. Neither choice implies that fn must be pure.

## Non-goals and explicit scope choices

- Runtime execution/firmware, while, recursion, mutation, break/continue, arbitrary iterators, I/O or environment-dependent evaluation.
- General Bool/if, branch-selection semantics, value-returning const fn, general dimensional algebra and automatic parts selection. Count-controlled topology is included through zero/one helper invocations, and through direct local declarations only in B. This does not introduce short-circuit branch checking, zero-length physical arrays or a parts-selection policy. General configuration still needs explicit branch checking/identity and device-fact contracts.
- New electrical assertions, inferred pin voltages/currents, a general constraint solver, simulation or new residual DRC rules.
- Multidimensional arrays, array-valued fn parameters, escaping iteration-local references, returned circuit collections, string interpolation or source identifier generation.
- Integer generics on device declarations, new pin-interface generation, global/exported const declarations, or general property introspection.
- A layout/router implementation, new coordinate frames or reach-in to fn-created instances. RFC-032 group transforms and internal overrides remain available; only B extends placement to direct lexical loop locals.

General-purpose TypeScript execution in tscircuit demonstrates useful separation of computation and object construction; it does not establish CoHDL's boundedness, unit rules or identity guarantees. The observed map-based capacitor construction is design evidence, not normative authority: [fixed source](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/projects/seveibar__rp2040-zero/lib/RP2040Circuit.tsx). Its preview/routing facilities belong to tooling/product comparisons, not an expansion of cohdl.dev's language responsibility.

Concrete comparison at core tag v0.0.1875, commit `c298605b2779793876f20f1139a2ebd024a1d7f3`: [unnamed components](https://github.com/tscircuit/core/blob/c298605b2779793876f20f1139a2ebd024a1d7f3/lib/components/primitive-components/Group/Group.ts#L678) use a per-kind counter within their subcircuit, while [selector fallback](https://github.com/tscircuit/core/blob/c298605b2779793876f20f1139a2ebd024a1d7f3/lib/components/base-components/PrimitiveComponent/PrimitiveComponent.ts#L1250) can enter an explicitly named subcircuit. These are different identity/access mechanisms from CoHDL's persistent designator allocation and port boundary; they do not establish that all explicit names drift or that tscircuit violates its own contract. The RP2040 example passes existing strings from a name list, without string interpolation. Its construction and external naming must be evaluated separately when comparing A/B. These are source observations, not executed upstream tests or a benchmark of either candidate.

## Design

### 1. Meaning: finite construction, followed by the existing checks

The three scenarios define the circuit-construction effects. A for expands its admitted body for each integer in a statically finite range. A/B share net/nc/helper operations; B additionally repeats direct inst/subdesign declarations. Every real component and connection still enters the ordinary shared checks. No loop instruction or fake container component reaches manufacturing IR. The rules below define one evaluator and expansion mechanism for those scenarios and their compositions.

### 2. One expression model with two domains

The new evaluator handles **Int**, a signed exact 64-bit structural integer, and the existing **Length** type. Int is not a twelfth electrical unit or an implicit quantity with a missing suffix. Its representable domain is −2^63…2^63−1; literals and every intermediate are checked. Zero and negative intermediate integers are valid; each consuming operation applies its own domain restriction.

| Operation | Result and rule |
| --- | --- |
| Int `+ - *` Int; unary `+ -` Int | Int, checked overflow; no wrapping or saturation |
| Int `/` Int | Int, quotient truncated toward zero; zero divisor and MIN/−1 are errors |
| Int `%` Int | Int, remainder satisfying a = (a / b) * b + r; zero divisor and MIN/−1 are errors |
| Length `+ -` Length; unary `+ -` Length | Length, exact existing fixed-point representation |
| Int `*` Length or Length `*` Int | Length, checked exact scaling; Int never becomes Length |
| Length `/` Int | Length only if exactly representable; zero divisor, overflow or lost precision is an error |
| Int `+` Length, Length `/` Length, Length `*` Length, other unit arithmetic | Rejected in this RFC |

Thus `10mm + n * 4mm` is valid; `10mm + n`, `3.3V + 1V` and `1mm / 0` are not. `1mm / 3` is rejected if it cannot be represented exactly, not rounded. Existing physical-coordinate range checks still apply after evaluation. There is no dependency on platform libm.

Length uses the existing **signed i128 count of 10^-15 mm** (`UnitValue.femto`); its expression intermediates are checked in that representation. The narrower geometry-consumer bound is applied at the existing consuming stage, not to all constants/generic values. An arithmetic-derived Length value gets canonical text from the existing `geom::mm_femto` convention: minimal decimal followed by `mm`, no redundant trailing zeros or negative zero. Thus `1.00mm + 0mm` produces value text `1mm`. A literal, parentheses around it, or pure parameter/constant forwarding preserves its original value text; arithmetic, including unary arithmetic on a value such as `-PITCH`, produces canonical result text. An adjacent prefix minus on an existing signed unit literal is assembled into that literal by the parser (§10): `-1.00mm` preserves its spelling. This is a literal/AST guarantee, not a requirement to retain a single signed lexer token. Source formatting retains the expression AST separately. No emitter may invent its own spelling or put an unevaluated expression into `UnitValue.text`.

Integer literals are decimal integers; no new radix or fractional spelling is introduced. A prefix sign directly applied to a decimal literal is combined with its magnitude before the Int range check, so `-9223372036854775808` is a valid literal/default and `9223372036854775808` is not. Other unary operations are evaluated normally with checked overflow: if MIN is already an Int value, `-MIN` fails. Parenthesizing a signed literal preserves it; parenthesizing the out-of-range positive magnitude does not make that magnitude valid. Test MIN, MAX, both adjacent out-of-range values, negating MIN, MIN/−1, and negative defaults explicitly.

Operator precedence, highest first: primary/parentheses → unary → `* / %` → `+ -` → range delimiter. Binary arithmetic is left-associative. Comparisons, booleans, casts and exponentiation are outside the grammar. Operations are evaluated in source-tree order; reassociation must not hide overflow.

### 3. Local constants and shared integer parameters

Local `const NAME: Int|Length = EXPR` is admitted in design, fn, subdesign, circuit-loop and layout bodies in both candidates. A const is immutable, creates no physical object and is not an exported declaration. It may use visible constants, Length parameters, const Int parameters, enclosing binders and visible array `.len`. It cannot inspect pin voltages, instance specs, registry state, files or generated placements.

`const N: Int` extends the existing RFC-007 generic list for **both fn and subdesign**. Calls retain `helper::<N>(...)`; subdesign uses retain `subdesign bank: Bank<N>` and `[Bank<N>; M]`. Their argument binding, defaults, nested forwarding, resolution and package visibility use one shared mechanism. Int defaults remain explicit integer literals; Int arguments and Length arguments admit the stated expressions. Other physical units keep their literal/parameter-only rules. No new array-valued or Int-valued runtime fn argument is introduced.

The contextual const marker distinguishes a structural value from an existing trait named Int: `N: Int` retains trait-bound meaning where such a trait exists. Integer generics on device declarations remain excluded and use a targeted E406 owner error; physical pin-interface generation belongs to its existing separate model. Unit/trait device parameters continue unchanged. Int is not an electrical spec value.

Constants and array lengths within a lexical body form an acyclic dependency graph, including subdesign array lengths. Forward references in the same body are allowed. Report the complete cycle for `const N: Int = channels.len` and `subdesign channels: [RcChannel<1kohm, 100nF>; N]`. Do not instantiate channels to discover their declared length. Child constants never escape to a parent or sibling; a layout constant is visible only in that layout and its descendants.

New constants, loop labels and binders cannot collide with or shadow visible instances, subdesign use names, arrays, nets, ports, fn parameters, generic parameters, constants or enclosing labels/binders. Check duplicates in the shared declaration validator. Existing same-net continuation stays valid. Labels belong to the containing lexical circuit body, with its layout blocks transparent for label uniqueness; binders belong to their own loop body. Disjoint sibling loops may both bind i.

### 4. One array evaluator, existing reference kinds

Physical and subdesign array lengths both accept Int expressions and remain strictly positive; zero/negative lengths remain E211. `.len` returns the declared length of a visible physical or subdesign array name without constructing its elements. A scalar has no `.len`. This is one query, not general member introspection: no `bank.channels.len` or reading a private const across the port boundary. Inside bank's own definition its channels array can be queried normally.

Every already-legal single-index position uses the same evaluator, including physical pins, subdesign ports, legal fn arguments, arrayed subdesign use sites, and **each indexed segment of an existing placement path**, e.g. `place banks[b].channels[i + 1].c at (...)`. Validate the concrete bounds of each segment against that node's own array. This extends selectors, not access rights: an electrical path cannot reach c through private internals, a whole subdesign cannot become an Instance argument, and a port cannot be marked nc.

Range/list fan-out remains net-member-only for physical pins and subdesign ports. The comparison baseline retains inclusive `..=` and its existing optional step form; bounds/list entries/step admit Int expressions, with positive step, start ≤ end and all selected elements in bounds. Never clip. Selecting many pins or ports contributes members to one authored net; loop iteration can instead contribute multiple distinct nets.

**Endpoint spelling is an explicit scope decision.** Half-open loops next to inclusive fan-out impose two conventions, independent of their electrical effects. Retaining inclusive fan-out minimizes changes to RFC-024. The alternative is to add half-open `arr[0..arr.len].PIN` alongside it, preserving old syntax. That alternative also needs empty-selection/member-count rules, step semantics, diagnostics and fmt tests; it is not implicitly included here. Decide this at acceptance, rather than claiming that one-net semantics explain the spelling difference.

### 5. Finite for and candidate-specific construction

For comparing the candidates, this revision uses the following **proposed labelled spelling**. The label policy remains a scope decision, as explained in §7.

```text
for_stmt       := 'for' LABEL ':' IDENT 'in' expr '..' expr '{' loop_stmt* '}'
const_stmt     := 'const' IDENT ':' ('Int' | 'Length') '=' expr
int_parameter  := 'const' IDENT ':' 'Int' ('=' integer_literal)?
primary        := literal | IDENT | IDENT '.' 'len' | '(' expr ')'
```

The half-open range a..b enters a through b−1 in order; equal endpoints are empty and reversed endpoints fail E1404. Bounds must be concrete Int before entry. Signed bounds and nested loops depending on an enclosing binder are allowed; a negative array index still fails. No inclusive for, for-step, mutation, break/continue or arbitrary iterator is added.

| Context | Candidate A | Candidate B |
| --- | --- | --- |
| Circuit loop in design/subdesign | const, net, nc, fn calls, nested for, placement-subset layout | A plus scalar/array inst and scalar/array subdesign use sites |
| Circuit loop in fn | Same operations; inherited fn placement restrictions apply | A plus scalar/array inst; subdesign use still E1307 |
| Loop inside layout | const, place, nested for | Same; no direct inst, subdesign, fn call, net or nc |
| Ordinary circuit body outside loops | Existing inst/subdesign/fn operations, plus M2 const/for/count expressions | Same |

Calling a fn from an A loop may create its ordinary local components. A is therefore an operation-body restriction, not a pure or wiring-only language. Fn's existing legal constraints/attributes remain available and all resulting effects are metered. No candidate adds top-level declarations/imports inside a loop or turns fn into pure computation.

For B, collect local declarations before that frame's operations, as existing circuit bodies do. Array elements are real and positive-length; a scalar contributes one occurrence per entered frame. A direct local subdesign in a design/subdesign loop has a logical node, ordinary ports and real children, using the same expander as a non-loop use site. The enclosing circuit owner is inherited; entering a loop inside fn does not authorize subdesign or place. B locals are visible only in their own iteration and lexical descendants, never through an external `loop_label[i].r` path. No same-name export into the parent is inferred. Nested loop dimensions do not add `a[i][j]` syntax.

A loop never declares an electrical interface or coordinate frame. A layout block inside a circuit loop is limited to const/place/nested for; ordinary layout blocks retain their existing constraint forms. Candidate B's directly declared subdesign may be placed as a whole or reached through its existing local path **from inside that lexical scope**. This does not create an outside-loop access interface. The separate-layout limitation remains a cost of choosing B's local form.

### 6. Placement preserves ownership and override semantics

```cohdl
layout {
    for grid: n in 0..leds.len {
        place leds[n] at (10mm + (n % 5) * 4mm,
                         10mm + (n / 5) * 4mm)
    }
}
```

`at` accepts Length expressions; rotate accepts Int checked against the live 0…359 rule; side remains literal top/bottom. Existing fixed-point transforms remain shared. No clearance, collision, routing or thermal solver, new angle type or pad/footprint expression language is introduced.

Placement context is inherited from the design/subdesign/fn owner, not recovered from a generated path string. In a design, computed coordinates use the board frame. In a subdesign, they are defaults in that subdesign's local frame, composed through the ordinary enclosing anchors, rotations and sides. An unanchored subdesign retains its existing no-default-board-placement behavior. A loop creates no additional transform or override priority. Fn→for→layout→place remains E1007; a board outline in fn remains E1006.

A and B can place a declared subdesign array or override an internal instance through RFC-032's existing dotted path. An outer instance override beats an inherited default for that instantiation. Two authored placements at the same precedence that resolve to the same target remain duplicates (E1007), even if one came from a loop. Iteration order cannot introduce last-write-wins behavior. The RC example places channel groups once and overrides one inherited capacitor default; it does not place that capacitor twice at board precedence.

B additionally allows a lexically visible direct loop-local instance to be placed in that loop or a descendant layout: board coordinates for a design-owned local, owner-relative defaults for a subdesign-owned local. This is an explicit extension of the placement declaration scope. It does not make B's private objects reachable from a separate outside layout or make fn locals placeable. Existing reachable subdesign children declared outside loops retain their interface unchanged.

### 7. Shared hierarchy machinery and loop identity

Name spelling and object kind are distinct. Current arrays derive element names with `element_name(base, i)`; subdesign and fn expansion extend `Scope.path`, and physical child paths feed the same designator allocator. M2 must extend that shared path construction/provenance mechanism, not build an independent allocator or infer a physical type from a string prefix.

| Source of identity | Existing or candidate path | Addressability |
| --- | --- | --- |
| Existing physical array element | `Board::leds_3` | Existing physical instance/pin contexts |
| Existing subdesign array child | `Board::channels_3::c` | Ports for electricity, existing dotted paths for placement |
| Candidate loop-generated operation/helper local | `Board::__for_wiring_3::LINK`, `Board::__for_decouple_3::__fn0_helper::c` | Operation provenance; fn locals do not gain placement access |
| Candidate B direct local / local subdesign child | `Board::__for_local_3::r`, `Board::__for_local_3::channel::c` | Lexical references within the iteration; no public loop reach-in |

The proposed `__for_LABEL_VALUE` segment is an internal lexical frame, not another subdesign node. It has no ports, part, designator, BOM row, default layout or implicit override priority. Real contained children use the ordinary mechanisms. Reusing a frame implementation does not justify inventing public loop objects. Conversely, giving loops public group access later would require an explicit interface/compatibility decision.

A loop that only wires or places a predeclared object **never reparents that object**. Extending the RC array preserves its old `channels_0`…`channels_9` child paths. Loop-generated nets and fn calls need distinct provenance even in A. For the labelled baseline, key frames by the loop label and actual integer value, not source-order ordinal or line number. Nonnegative values use decimal text, negative values use neg followed by the magnitude. Reserve the __for_ family through the existing reserved-name rule and test injectivity against nested arrays/subdesign/fn paths.

Each iteration owns its fn-call and anonymous-net counters and does not consume its parent's legacy counters. Net declarations within an iteration use its lexical scope: equal local spelling across iterations does not short nets; ordinary same-net continuation within a frame remains. Explicitly shared pins/ports merge connectivity through existing rules. No private net escapes by printed name.

Reordering differently labelled loops and extending a range preserve surviving label/value paths. Renaming a label intentionally changes generated paths. The existing ordinal identity limit for fn calls *within* a frame remains: reordering those calls can reassign their purposes without a helpful lock diff, so compare connectivity/parts/layout too. Duplicate labels fail E201; repeated hard-coded designator overrides still fail normally.

**The mandatory-label cost is not settled by this mechanism.** The labelled baseline names every loop, including pure placement loops, to avoid prematurely inventing effect inference or anonymous-site matching. An optional-label alternative must explain identity for named/anonymous nets, repeated effectful helper calls and edits that later add a constructor; placement-only loops may need provenance without persistent object identity. Compare these costs before acceptance. No claim that shorter syntax must sacrifice correctness, or that the baseline spelling is uniquely required, is made.

### 8. Uniform declaration checks, concrete binding checks and final assembly

Use one declaration validator for all supplied fn and subdesign definitions, independent of whether they are called, contain M2 syntax or are reachable from the selected design. Apply the same statically decidable checks to new loop scopes. Check symbol/kind resolution, lexical duplicate/shadowing names, generic argument arity/default kinds, unit compatibility and trait/reference constraints when the typed declaration environment establishes the necessary facts. A constraint depending on an unknown concrete device or value waits for substitution. A caller's unused const must never switch these checks on or off in a callee.

This deliberately **replaces the previous new-loop-only strengthening policy**. The baseline `check/bodies.rs` documents gaps such as uncalled duplicate-local and abstract generic checks. Closing the statically decidable duplicate/generic/reference gaps uniformly is part of this proposed amendment, not an unnoticed implementation refactor. Its compatibility cost is explicit: an old source containing such invalid but previously unchecked uncalled definitions may now fail. Update the ledger and include isolated before/after tests. Do not silently broaden this into new universal fn recursion or numeric proofs: RFC-006 active-call cycle rejection remains, and RFC-032 recursive containment remains declaration-time with its full cycle diagnostic.

**Compatibility evidence required before acceptance:** run the existing checker and a prototype of the proposed declaration validator over every package in `lib/`, with the same recorded source revision and exact dependency set. Include unused fn/subdesign definitions; checking only instantiated board examples cannot measure this change. Report packages/definitions examined, newly rejected definitions and diagnostics by category (including a measured zero), and the source locations and migration needed for each difference. Record unresolved dependencies or skipped definitions explicitly; they cannot count as passing coverage. Preserve the pre-change valid corpus's verdicts and emitted bytes except for individually documented new declaration errors. This corpus audit has not run: the proposed validator is not implemented. A passing run of today's `cargo test` is not evidence of this future compatibility cost.

Use one typed evaluator in three environments:

1. **Definition validation:** parameter and binder types are known, values may be unknown. Resolve symbolic obligations and evaluate concrete subexpressions. An uncalled `SeriesR<C>` for C: Capacitance fails without needing a C value. An empty B loop cannot hide a duplicate inst or known unit mismatch; A rejects the forbidden direct declaration by context instead.
2. **Actual fn or subdesign activation:** bind arguments/defaults, then validate the complete body under that substitution before its children are materialized. Values depending on those arguments become concrete, including in an empty nested loop. A declared subdesign array activates each actual element normally.
3. **Actual iteration:** bind that integer, validate newly concrete expressions and contribute the selected candidate's operations. Repeat for nested calls/use sites/loops. Type-only validation allocates no hypothetical objects or dynamic loop frames.

A known zero divisor is a failure even if its numerator is unknown: i/0 fails inside an empty loop. 1/i waits for an actual i. Do not invent unknown values by algebraic simplification. Constants are evaluated in deterministic dependency order and memoized per typed/bound environment, never across incompatible bindings.

Calling `f::<0>()` where f contains `for empty: i in 0..0 { const BAD: Int = 1 / N }` fails under the actual binding. `for none: i in 0..0 { f::<0>(); }` makes no actual f activation: its signature and argument expressions are checked, but no callee value specialization occurs. `f::<1 / 0>()` there still fails at its own argument expression. The same activation boundary applies to B's skipped subdesign use sites. A successful empty loop contributes no parts, logical nodes, ports, nets or placements.

After expansion, perform existing connection merging, required-port validation, physical-pin obligations and residual DRC; build retains part/footprint/designator obligations. A helper may leave caller-owned pins to its caller; temporary port members must remain until required-port checking and merging finish. Loop expansion cannot erase a required port or treat it as a physical nc. Errors retain original source, actual bindings and fn/subdesign/iteration provenance through downstream checks.

### 9. Deterministic work limits across the full expansion graph

Budget activation and generated-work counting are separate from declaration type checking. First walk the **complete syntactic expansion-reference graph rooted at the selected design**. Follow every fn call and subdesign use-site type, including nested subdesign arrays, helpers called by subdesigns and references lexically inside loops. Inspect each reached definition's signature/defaults and body with a visited set. Device/part references do not contribute executable bodies; simply declaring an unrelated function/subdesign in a package does not make it reachable. Loop trip counts and const liveness do not prune this syntax walk, and the walk does not instantiate definitions.

Activate metering if this graph contains an admitted M2 const, const Int parameter, loop, `.len` or arithmetic expression. Literal old array syntax and ordinary unit-parameter forwarding alone do not activate it. Thus an Int default in a reached subdesign activates it, as does a reached-but-skipped helper containing M2; an unrelated uncalled M2 definition does not. Recursion is not enabled by the visited set: existing fn and subdesign cycle rules remain independently enforced at their stated stages.

**A concrete regression is required:** selected design → subdesign wrapper → helper whose body uses M2, with no direct fn call or M2 syntax in the selected design. Metering must activate before the first generated object. Also test nested subdesigns with no fn and M2 only in the deepest body. The prior fn-only discovery rule is removed, not left as a future integration question.

A purely legacy selected graph keeps its old resource-limit behavior. That compatibility exception is not a promise to cap every old large subdesign board. Once metering is activated, it covers **the whole actual expansion**, including earlier legacy declarations and descendants of all fn/subdesign activations. Source reorder, dead-constant elimination and optimizer caching cannot change activation or counts.

Limits are 100,000 cumulative entered loop iterations, 1,000,000 generated work items, and 64 active loop frames (including across fn/subdesign activation). A logical node is not itself a loop frame. These are proposed language constants, not host-memory heuristics or wall-clock timeouts; exceeding a limit is a deterministic hard error before appending/materializing the excess item.

| Actual semantic event | Work items charged before materialization |
| --- | --- |
| Physical scalar/array element | 1 per real instance |
| Fn invocation | 1 per entered call, plus all its contributed items |
| Subdesign scalar/array element | 1 per logical node, plus its declared ports and contributed items |
| Instantiated subdesign port | 1 per port, including optional/disconnected ports; logical work, not a BOM component |
| Authored port-connection-block entry | 1 per entry plus 1 for its scalar target reference; the synthesized join does not additionally count as an authored net |
| Authored net/nc/place statement | 1 per expanded statement; net/nc additionally charge 1 per expanded member, before deduplication |
| Layout constraint | 1 per expanded declaration plus 1 per authored net reference |
| Physics attribute / diff-pair physics bracket | 1 per resolved record plus 1 per authored pin/instance target; a diff-pair bracket is in addition to its constraint |

The existing prohibition of one port block on a whole subdesign array remains. For the allowed scalar block, targets are scalar pin/port/net references; fan-out is still net-member-only. Internal required-port checks and union-find steps add no optimizer-dependent charge. Count references as specified, not every path segment traversed. Derived geometry/output rows do not silently become new work events.

Array bases, const declarations, layout blocks and loop headers add no separate work item. Logical nodes **do** count even when empty, so empty nested subdesigns cannot amplify unmetered expansion. Port declarations **do** count even when no real component is inside. Duplicate authored members count before merging. Physical descendants are charged regardless of whether reached through a helper, subdesign or B's direct local constructor. An empty loop still contributes no generated work.

Preserve ordinary declaration-before-operation traversal, using declaration source order and entered integer order recursively; then charge authored operations and their member/reference lists in order. Deferred placement/port operations must be charged before their records are allocated. Determine large counts with checked arithmetic and reject over-budget requests before bulk cloning/allocation, while preserving the same first failing source site as incremental charging. Do not allocate a million hypothetical objects just to count them. Optimization cannot change the first failing event, verdict or reported count/limit.

Worked counts: two real one-pin instances, two nc statements with one member each, and two place statements cost 8 work items and 4 iterations when emitted by two two-iteration loops. A board with one unused M2 const and `subdesign nodes: [Empty; 2]` costs 2 work items even if Empty has no ports or physical children. A scalar one-port subdesign with one internal resistor and one authored two-member net costs 1 node + 1 port + 1 instance + 1 net + 2 members = 6 before its caller's contributions; final connection obligations still apply. These distinguish resource work from manufactured parts.

The reference walk selects metering only; §8 checks declarations uniformly without a second feature-propagation policy. Neither pass evaluates unentered value specializations. These limits bound the stated expansion events, not source parsing, every legacy resource use or downstream geometry emission. Constant dependency cycles are rejected; memoization prevents exponential repeated constant evaluation. Failed expansion emits no new successful or truncated build and preserves the existing artifact failure/ownership policy.

### 10. Diagnostics and lexer compatibility

Reuse established errors where their meaning applies: E201 name collision, E202 resolved index out of bounds, E211 invalid array shape/length, E401 generic arity, E403 trait bound, E110/E111 physical-unit mismatch/missing unit, E112/E113 unit generic arguments, and the existing placement/DRC errors. The new generic marker's unsupported owner uses E406. The E201 registry wording must be updated from "duplicate top-level declaration" to cover the existing lexical-scope collisions as well as new loop names; `expand.rs` already emits E201 for these. This is an explicit documentation alignment, not an unallocated new error meaning. Operator-domain failure uses E1401; a physical-unit mismatch at a consuming field uses its existing E110/E111 or generic E112/E113. A Length expression where Int is required uses E1401. Newly supported negative Int expressions can reach E202/E211 bounds checks instead of the old lexical E102; record that intentional diagnostic refinement, while keeping E102 for negative bare-number consumers that still do not admit Int.

Proposed new block, subject to central allocation at acceptance:

| Candidate | Failure kind |
| --- | --- |
| E1401 | Expected compile-time Int/Length or a supported operand pairing; non-integer number, wrong kind or unsupported property |
| E1402 | Integer/Length expression overflow or non-exact Length division |
| E1403 | Division/remainder by zero |
| E1404 | Reversed for range |
| E1405 | Deterministic elaboration budget/depth exceeded |
| E1406 | A declaration/operation is not admitted in this loop or expression context |
| E1407 | Cyclic constant/array-length dependency, naming the complete cycle |

The primary span is the smallest failing original construct. For an expansion-dependent error, the **main message** must carry the violated constraint, concrete failing value/valid bound, and a compact distinguishing activation path with relevant bindings, such as `Board::bank::__for_links_9, N = 10, n = 9`. Essential context must survive consumers that display only code/message/primary span. Secondary labels add enclosing fn-call/subdesign-use and loop-header spans and declarations establishing the bound. Preserve RFC-010 schema_version 1 and existing message/secondary/help fields. An expansion error's internal deduplication key must include the expansion frame, so two distinct failing iterations are not silently conflated merely because their source span matches. No hidden diagnostic truncation is introduced.

The evaluator's Int domain must not weaken rejection of a bare number in a Voltage or Length position. `rotate` remains a separate existing consumer of integer values, not a physical-unit coercion.

Lexing requires deliberate compatibility checks: `for` is already a token for `impl Trait for Device`; `const` and `in` are contextual in their new grammar positions, not globally reserved identifiers. An ordinary missing operator must not become implicit multiplication.

**Minus has one lexical spelling.** Outside strings/comments, every `-` is a standalone punctuation token, including before digits; the numeric token following it carries no sign. Remove the current lexer branch that absorbs a following number, rejects negative bare numbers with E102, or rejects other uses of `-` with the claim that only Temperature/Length signs are allowed. Numeric spelling, suffix and source spans remain available to the shared literal reader; signed-unit validation must preserve the existing rules and diagnostics, even if validation moves out of the scanner. Internal token shape is not the source/IR compatibility promise.

**The parser determines the role within the admitted grammar.** In an expression's operand position, `-` is prefix; after a completed left operand it is binary subtraction, with §2 precedence. Thus `n-1` and `n - 1` have the same expression tree; `-PITCH` is prefix arithmetic, and `n--1` is subtraction of a negative literal. A parsed loop header consumes contextual `in` before starting its lower-bound expression, so `in -1..N` begins with a prefix sign. Do not classify minus globally from the preceding token kind: `in` is also an ordinary identifier outside that header. Legacy literal-only fields do not acquire expression grammar through this change.

**Literal assembly preserves signed-unit compatibility.** Where the grammar expects a literal or a prefix operand, a minus immediately adjacent to a numeric unit spelling forms the existing signed literal, including its original full text and sign-inclusive span. Use the same literal validation for all consumers: `-1.00mm` and `-40C` retain their values/text; `-5V` remains E105. The unsigned operand of binary subtraction is not a negative literal: `1mm-1mm` subtracts two positive Length literals. A space, comment or parenthesis between a prefix minus and its unit spelling prevents literal assembly. In an admitted Length expression, `- 1.00mm` and `-(1.00mm)` are unary arithmetic and produce canonical `-1mm`; the existing spelling `-1.00mm` remains literal text. Fmt must preserve this AST distinction across reparsing, as well as literal spelling. This does not add Temperature arithmetic: `- 40C` and `-(40C)` remain invalid. Unit suffix, precision, range and unsigned-negativity checks are not relaxed.

For decimal integers, §2's direct-sign range rule applies in Int grammar, including literal-only Int defaults; a space or comment does not introduce an intervening expression node, but parentheses do. Keep E102 at legacy negative-bare-number consumers; admitted Int expressions instead receive the stated expression/bounds diagnostics. Merely removing the lexer error must not make negative bare electrical values valid. Regression tests must cover old generic defaults/arguments, specs, coordinates and other literal consumers, not only new const expressions.

Keep the current maximal unit-suffix rule: **`10%` is a Tolerance literal; `10 % 3` uses remainder**. `10%3` is rejected rather than reinterpreting a unit literal based on an expected type; fmt always puts spaces around binary operators. Test all sign cases above, Int MIN/defaults, binary subtraction followed by unary signs, `0..10`, `0..=9`, `//` comments, and targeted E101/E102/E105/E106 diagnostics. Add parse → fmt → reparse checks for literal-versus-unary Length text and sign-inclusive spans. No symbol-table-dependent tokenization or unbounded lookahead is allowed.

## Type-system-first test

This RFC adds no rule/DRC check. Unit substitution and trait/Pin-vs-Instance requirements remain type-level obligations. Int range/index/overflow checks and finite generation limits are compile-time elaboration obligations. Required physical pins and required subdesign ports are structural obligations checked at their existing final-assembly stages because only then is caller connectivity known. Existing emergent electrical checks remain residual DRC.

The preservation criterion is concrete: every generated instance carries the same resolved device, specs, pin roles/obligations, trait bindings and eventual part requirements that an equivalent hand-written instance would carry. Generic or repeated construction may not replace a failed check with an unchecked placeholder, erase a required pin, or treat missing data as success. An editor may retain a partial model under its existing error state; it is not a successful build.

Three paired negative probes are mandatory: wrong unit in an RC parameter (also through nested forwarding); an out-of-range LED reference with a well-typed Int; and a missing required connection on an actually generated channel. Add a fourth probe that deliberately cross-connects still-well-typed channels: the topology oracle must catch the mismatch without falsely claiming the language can infer intent. This ties the examples to the verdict ladder rather than merely counting generated objects.

## Conceptual impact and explicit amendments

The permanent new capability is **typed parameterized circuit construction through bounded compile-time evaluation**. Its shared conceptual cost is the pure-value/effect distinction plus hygienic iteration frames. Candidate B additionally pays for direct local declarations and their restricted external addressability; the comparison must justify that cost. For is one consumer; array selectors and const-generic arguments are others. Do not create separate "array arithmetic", "layout arithmetic" and "module programming" engines.

The review guide's P01–P10 are traceability labels for existing principles, not new normative authority:

| Principle | Concrete consequence in this RFC |
| --- | --- |
| P01/P02: checked promises, earliest sufficient information | Definition/typed-binding checks, concrete index validation, then final pin obligations; paired negative examples are mandatory |
| P03/P05: shared semantics, explicit sources | Reuse inst/subdesign/fn/net and generic binding; sharing a pin explicitly merges nets, equal local spelling does not |
| P04/P06: fixed inputs, persistent identity | Pure evaluation, exact locked dependencies, labelled iteration paths; repeat-build bytes and edit stability are separate tests |
| P07/P08: physical responsibility, tooling | Every generated part remains in the pipeline; format, diagnostics, LSP and symbolic docs ship with the feature |
| P09/P10: bounded claims, explicit evolution | Disclose intent/analog/physical gaps and deferred capabilities; amend affected Accepted contracts only through acceptance |

| Existing contract | Required explicit amendment / preserved boundary |
| --- | --- |
| RFC-001 units | Int is outside the physical unit set; exact Length operations reuse its representation and zero-coercion discipline. E102 must explicitly cease rejecting negative Int literals in the newly admitted integer contexts |
| RFC-007 generics | Amend both the exclusion of non-unit const generics and the literal/parameter-only restriction on Length arguments. Add `const N: Int` for fn/subdesign and Length argument expressions through shared substitution; defaults remain literal-only |
| RFC-024 arrays | Replace literal-only counts/indexes with checked Int expressions; retain positive length, real elements and net-only fan-out |
| RFC-032 subdesign | Preserve typed ports, hierarchy and layout contracts; explicitly extend count/index computation, local const/for, shared declaration validation and full expansion-graph metering |
| RFC-006 fn | Preserve effectful calls and real locals; share parameter binding/provenance; retain active-call cycle rules; explicitly strengthen statically decidable declaration validation |
| RFC-005 identity | Existing array paths and allocator unchanged; specify loop-generated paths and disclose edit-induced identity changes |
| RFC-009/010/011/014 tooling | Canonical formatting, span/provenance, registered errors, LSP understanding must ship together |
| Package API-docs contract (`docs/apidocs.md`) | Version the symbolic body/signature representation explicitly; see Tooling. Diagnostics JSON remains version 1 |
| RFC-013/020/025/026 layout | Evaluate positions/rotations through existing geometry rules; inherit design/subdesign placement context; B additionally extends place to visible direct loop locals. Fn-local reach-in and pad expressions remain excluded; no routing or new electrical inference |
| RFC-029/030 dependencies | Offline check/build and exact locked packages unchanged; expressions cannot consult registry/network state |

This stays inside cohdl.dev's language/compiler responsibility. cohdl.ai may generate this source and consume verdicts; it must not supply a second preprocessor, privately interpret loops, or choose different arithmetic/limits. Konnect/KiCad operate on the compiler's outputs, with their normal physical checks.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
| --- | --- | --- | --- | --- | --- | --- |
| High | High | High | High | High | High | High |

- **Concepts:** justify Int/const by count-dependent real arrays and finite iteration. Explicitly exclude runtime values and physical spec arithmetic. Keep modules, subdesign, fn, Instance and Net meanings distinct; compare rather than presume a need for B.
- **Grammar:** one precedence table and expression AST; const generics extend the existing list. One loop range form in the comparison baseline; explicitly review the endpoint-spelling cost of retained inclusive fan-out. Contextual spelling avoids stealing existing declaration names.
- **Oracle:** static body validation plus concrete per-use elaboration, followed by the existing pipeline; no ignored statements, hidden unknowns or skipped real parts.
- **Diagnostics:** original spans plus expansion frames survive through existing check/build JSON and LSP; negative/empty/overflow tests are required.
- **Netlist:** loop-local names cannot short sibling iterations; no fake loop components. Check graph equivalence against a hand-expanded design, including all pins, NC and placements.
- **Compat:** pre-RFC valid-source golden outputs and design.lock must be byte-identical except the declared uniform-validation correction for previously unchecked invalid definitions; signed/unit lexing and legacy fn counters need specific regression coverage. Refactoring source into a labelled loop is a design edit and can change scoped net/fn-local names; do not falsely promise byte equality across that rewrite.
- **Trust:** deterministic failure on bounds/resource violations, no time-dependent evaluation or partial success; demonstrate both safe repetition and visible failure.

This is not a Low-impact parser convenience. Acceptance must update the affected normative sections and decision record in the same design change. Implementation cannot claim RFC-007/024 are unchanged while relying on their superseded restrictions.

## Gradeability and exit criteria

`docs/proposals/fixtures/m2-programmability/` contains historical current-syntax references and `.cohdl.txt` proposed counterparts. `rewrite-2026-09-10.json` retains the compiler/source hashes and actual observations from that earlier revision: LED 11 instances/12 nets; independent-input three-channel RC 13/7; shared-input nested RC 11/5. Wrong-unit, missing-pin and wrong-kind probes failed. Deliberately shorting two otherwise typed outputs passed check while reducing the independent RC net partition count from seven to six. Those are source-specific observations, not executions of this revision or proof that B is necessary. Earlier direct-local `.cohdl.txt` files remain Candidate B comparison material and must not silently become the current selected syntax.

Compare each candidate to an explicit oracle with the same device/spec/part choices, physical endpoint partition, nc decisions and effective placements. The RC workflow and nested bank differ in their input interface, so their distinct net counts are expected. Compare physical graphs under a stated instance correspondence when candidate paths differ. Do not compare incidental anonymous-net names as if they were topology, or require different source spellings to emit identical bytes.

The RC workflow additionally needs real parts: build ten channels, adjust only capacitor seven, then expand to twelve starting from that same lock. Preserve all surviving physical paths/designators/placements and the connectivity partition restricted to surviving endpoints, while accounting for new shared-ground endpoints. Compare complete BOM and part bindings, not just instance counts. Add repeat-build byte checks with compiler/version/options, source paths, dependency content/locks, prior design.lock and referenced resources fixed. Literal current-syntax fixtures establish the existing subdesign baseline; they do not test the proposed evaluator, loops, uniform validator or resource meter.

**Executed existing-syntax baseline (2026-09-10):** `rc-workflow/` in the fixture directory contains three literal sources and `verify.py`, checked with main `0e3d770` using exact locked 1kohm resistor, 100nF capacitor and connector parts. The ten-channel, capacitor-seven override and twelve-channel stages all build successfully, with repeat-build artifact/lock hashes equal at each stage. Surviving designators and connectivity restricted to old endpoints remain; only capacitor seven moves during its override, and no old component moves during growth. Ten/twelve channels contain 30/36 physical components, 20/24 passives and 21/25 nets. This fixture uses one real connector per channel instead of the synthetic input/output/ground interfaces above, so its total component count is 3N, not 4N+1. `results-2026-09-10.json` preserves the earlier run. The later `book-results-2026-09-10.json` records the verified compiler source commit/version, executable/input/source/verifier hashes, placements and five teaching probes; it also verifies primary BOM fields and unchanged dependency locks. These evidence updates do not retroactively change the earlier report. These results establish the existing subdesign editing baseline, not Candidate A/B execution or physical clearance/production approval.

Candidate-specific tests become implementation requirements only if that candidate is selected. Shared requirements must ship together:

| Case | Required result |
| --- | --- |
| LED N = 1/2/10 | 0/1/9 neighbor nets, shared power nets, explicit input/final nc, ordinary pin checks |
| Physical/subdesign array zero, negative, non-Int or overflowing count | Error before array element/node materialization; no zero-length hardware-array shortcut |
| Wrong R/C through nested subdesign/fn; unsatisfied trait; Instance versus Pin versus logical node | Existing typed binding distinctions and targeted diagnostics with activation provenance |
| Missing required port / internal required physical pin / nc on a port | Existing distinct structural errors; a container never conceals an obligation |
| Independent RC versus shared-input nested bank | Each matches its own oracle; no false equivalence between different interfaces |
| Ten-channel independent layout → capacitor-seven override → twelve channels | Surviving paths/designators/positions and restricted endpoint partitions preserved; added real parts fully accounted for |
| Computed selectors in `banks[b].channels[i].c` placement | Each segment checked against that node's actual array; electrical reach-in remains rejected |
| Subdesign relative layout, nesting, rotation/bottom and individual override | Same effective transforms and precedence as explicit current syntax; no loop coordinate frame |
| Duplicate direct placement from a loop and explicit statement | E1007; no last-write-wins; legitimate inherited-default override remains valid |
| Scalar/array constructors in A loop / B loop | A: context rejection; B: ordinary typed local construction with no outside-loop reach-in |
| B nested subdesign local, empty loop, optional ports and real descendants | Zero iterations create nothing; entered nodes/ports/descendants receive normal checks and budgets |
| Fn→loop→subdesign or place; board outline inside fn | Existing E1307/E1007/E1006; a loop cannot change the fn owner |
| Uncalled duplicate-local or known generic mismatch, with/without M2 const in caller | Same declaration error in both sources; explicit correction to prior unchecked-invalid behavior |
| Empty body with unknown symbol or invariant `i / 0`; skipped `1 / i` | Static failures where decidable; no invented iteration value |
| Actual fn/subdesign binding with `1 / N` inside an empty loop | Concrete binding error; skipping that activation does not specialize its body |
| Skipped `f::<0>()` versus `f::<1 / 0>()` | No callee specialization versus immediate invalid argument expression |
| Active fn cycle; recursive subdesign containment | Existing respective stages and full cycle diagnostics; no recursion enabled by budget discovery |
| Int/Length mismatch, extrema, intermediate overflow and inexact Length division | Exact-domain failures with original spans; no coercion, reassociation escape or rounding |
| Constant/array-length cycles, forward references, nested shadowing | Complete cycle/name errors or deterministic dependency evaluation without hypothetical allocation |
| Both array kinds `.len`; scalar or private `bank.channels.len` | Shared valid length query or targeted invalid query; no new data reach-in |
| Reversed for, inclusive fan-out boundaries, zero step | Declared endpoint/step errors; explicit tests for inclusive versus exclusive ends |
| Two labelled loops reordered/extended; a label renamed | Surviving frame identity retained or explicit generated-path change; parent arrays never reparented |
| B direct locals, local subdesign children and repeated helper locals | Shared injective path construction/allocator; each real part retained; containers no BOM rows |
| Reordered fn calls within one loop; repeated hard-coded designator override | Disclosed legacy ordinal limitation / ordinary collision rejection |
| Design→subdesign→M2 helper, or M2 only in deepest subdesign | Full graph enables metering before any object materialization |
| Pure legacy subdesign graph / unrelated uncalled M2 definition | No new meter; uniform declaration corrections still apply independently |
| Reached-but-skipped M2 helper, unused const or Int default | Deterministic whole-design meter activation, without callee instantiation during discovery |
| Empty subdesign array, port-only nodes and connection-block entries | Logical nodes/ports/entries count even without physical parts; no double charging synthesized joins |
| Exactly/one over 100,000 iterations, 1,000,000 items, 64 loop frames | Deterministic separate boundary tests with source/count/limit; no partial successful output |
| All declared work-event kinds, duplicates and large bulk counts | Charge before materialization in specified order; dedup/optimization cannot change costs |
| Const Int and Length defaults/forwarding across packages, fn and subdesign | One binding mechanism; existing exact dependency hash checks remain |
| Existing trait Int, identifiers const/in and old signed/unit literals | Existing valid meanings preserved; unsupported Int owners rejected explicitly |
| `10%`, `10 % 3`, `10%3`; fmt parentheses and fixed point | Existing Tolerance literal preserved, spaced remainder parsed, ambiguous no-space form rejected; fmt preserves AST/meaning |
| CLI/JSON/LSP/Explorer diagnostics with related information hidden | Main message still distinguishes fn/subdesign/iteration activations at one source span |
| Docs v2 symbolic fn/subdesign/design → registry → viewer | Accurate signatures/ports/canonical source; no invented expanded counts or unsupported-version downgrade |
| Old valid-source golden artifacts and intentionally stricter declaration cases | Bytes/locks/verdicts stable except the explicitly enumerated previously unchecked-invalid cases |

No proposed M2 behavior is claimed executed. Acceptance must select a candidate, settle the label/selector choices, update normative documents and implement the corresponding tests; merely marking the missing work as an open issue is not implementation readiness.

## AI-generatability

Teach the existing object model first: physical arrays name components, subdesign arrays name reusable ported regions, fn expands a typed fragment, and loops repeat admitted operations. The same evaluator computes counts, selectors and coordinates. The compiler reports actual binding/index/context at the original source span. Candidate B offers a second way to author private repeated contents; its benefits must outweigh teaching its placement/access limits.

The repair task is concrete: shorten a neighbor loop from `0..leds.len` to `0..(leds.len - 1)` after the final index fails. An in-bounds connection to the wrong channel can still be type-correct, so independent topology checks remain necessary. The RC layout task tests whether a later agent can identify and adjust exactly one component without rebuilding its construction abstraction.

## Alternatives and selection criteria

1. **Candidate A: operations over named physical/subdesign objects, with effectful fn calls.** Both kinds of arrays accept computed counts/indexes; subdesign/fn share Int parameters and constants. Circuit loops repeat net/nc/calls; layout loops place existing named objects. Ordinary declarations remain in fn/subdesign/design bodies. This is the comparison baseline and expresses the three shown workflows. It may require extracting a reusable fragment when a user wants private per-iteration creation; that cost needs a real example before it is judged unacceptable.
2. **Candidate B: A plus direct loop-local inst/subdesign declarations where their owner permits them.** Supports local construction without extraction, including per-iteration specialization. It preserves real parts/checks, but adds declaration scope, identity and limited external placement/access. Its direct-local RC form cannot alone satisfy the separate-layout workflow; using A's subdesign form under B does not establish a need for B. **The previous recommendation is withdrawn.**
3. **Pure fn plus subdesign-only construction.** A separate language-role and migration proposal, not a consequence of A. Account for Instance/trait-bound helpers, physical attribute targets, nested generated parts and lock/source compatibility. Current fn remains effectful under both A and B.
4. **Host-language interpretation or general-purpose computation now.** Offers broader calculation/control flow but requires contracts for effects, termination, packaging and diagnostics that are outside this M2 slice. tscircuit is evidence about authoring ergonomics, not proof of CoHDL correctness or identity guarantees. Its map example combines repeated construction with string-valued names that other declarations can reference. CoHDL excludes source identifier generation and escaping loop-local references, so that external edit mechanism cannot be assumed for B's private locals. This prevents using the example alone to justify B for a later external placement task; it does not disprove the usefulness of local construction within its admitted scope. Existing named physical/subdesign arrays remain available to both candidates.

Choose A or B only after comparing identical topology, external edit tasks, generated identity and compiler/tooling cost. An additional construction workload must identify the concrete limitation of A, such as per-iteration specialization that cannot be represented by its shared array/fragment model without material complexity. A shorter source snippet or a physically equivalent but externally uneditable graph is insufficient evidence. This revision specifies candidates and common contracts without inventing a workload winner.

Mandatory labels and half-open fan-out remain explicit spelling decisions, not consequences of choosing B. Uniform declaration validation is a proposed shared amendment: it replaces permanent feature-dependent check-strength exceptions, with the compatibility cost listed below. If that strengthening is rejected, acceptance must specify an equally explicit uniform alternative or stage the validator correction separately; it cannot silently restore caller-dependent checks.

## Compatibility

This explicitly amends accepted exclusions, rather than pretending all prior text remains normative. No existing array spelling is replaced. Existing valid source using none of the new constructs must preserve verdict, design.lock and every emitted byte, with one explicitly proposed correction: statically invalid uncalled definitions previously missed by the declaration checker now fail uniformly under Design §8. Pin down those newly rejected cases rather than claiming unconditional preservation of every prior successful check. Previously invalid syntax now admitted as integer expressions follows its newly specified diagnostics; this is not a promise to keep rejecting the new feature. Pin numbers, signed physical literals, trait parameters named Int, negative-number errors in old physical consumers and tolerance `%` literals need regression coverage beyond the happy-path loop fixture.

**Whitespace-sensitive `%` is an explicit lexical tradeoff.** Existing `10%` remains a Tolerance literal; `10 % 3` becomes integer remainder; `10%3` is rejected, not reinterpreted using expected types. This adds a whitespace-sensitive operator boundary while preserving old unit tokens. Fmt spaces binary operators, but does not make the authoring cost disappear. Test tokenization, completion and round-trip formatting; acceptance must acknowledge the tradeoff or choose a different operator spelling explicitly.

**Range and diagnostic compatibility:** retained inclusive fan-out coexists with half-open for in the comparison baseline. Adding half-open selectors would be an additive RFC-024 change with separately specified empty-range rules. The existing broader use of E201 must be reflected in `docs/error-codes.md` when updating the registry, and old E130x port/container distinctions are preserved.

Adopting loops may rename anonymous/local nets or fn-local paths. Preserve topology and existing array identities, inspect the lock diff and reconcile layout mappings. Do not automatically migrate routed boards or rewrite all source with fmt. Pure-constant arithmetic introduces no nondeterministic dependency inputs.

## Tooling & operations

Suggested implementation seams, not an implementation authorization: one expression AST/evaluator; the shared generic substitution environment gains Int; selector ASTs retain expressions until concrete resolution; body/layout walkers retain loop and const nodes rather than text-replacing source. Carry the existing Design/Sub/Fn placement owner through lexical iteration frames; do not replace subdesign-relative defaults with board coordinates. Preserve original spans and expansion provenance in shared checked data used by downstream diagnostics.

`cohdl fmt` formats, never unrolls or constant-folds authored source. Verify semantic preservation as well as idempotence: dropping parentheses from `(n + 1) * 2` can produce a different, still in-bounds circuit. LSP identifies constants, loop binders and array declarations, and shows concrete lengths/values where a single instantiation determines them; generic/loop references may have multiple values and must not show an arbitrary one as universal. LSP clients without relatedInformation and Explorer's existing message-only diagnostic projection still receive the essential expansion context specified in section 10. Explorer consumes the same expanded instances and original source mapping; there is no second JS elaborator.

**Package API docs need an explicit versioned contract.** Baseline `body_summary` emits flat insts/calls/nets and numeric array lengths (subdesign uses already carry a kind marker in the inst list); it cannot truthfully describe an uninstantiated N-element loop by reporting zero nets or inventing a length. The proposed minimum is:

- A document requiring M2 signatures/bodies uses package API-docs `schema_version: 2`; version 1 remains for documents whose emitted items need no new representation. This is independent of diagnostics JSON, which stays at version 1. Existing version-1 field semantics remain unchanged.
- In version 2, an Int generic uses `bound: {"const": "Int"}` in its existing generic descriptor; `name` and literal-text `default` keep their meanings. Unit/trait descriptors retain their existing forms.
- Every fn/subdesign/design item whose own signature or lexical body uses M2 has a `body_source` string in its kind-named payload. This is the complete braced body in canonical CoHDL formatting, preserving constants, symbolic lengths, nesting, labels and expressions. Omit that item's legacy insts/calls/nets summary fields rather than presenting incomplete direct-statement counts as totals. Retain the subdesign port descriptors and their obligations; other item fields retain their existing meanings. Use the same formatter; do not build a second body renderer or expose a competing executable JSON language.
- An M2 value expression in an existing generic-argument string list uses canonical authored expression text, not a guessed value. Derived concrete spec values use section 2's value-text rule. If emitted local or foreign items require any of these forms, the enclosing document is version 2.
- Registry upload validation/storage and the viewer must support both versions before version-2 uploads are claimed supported. In particular, update the existing version-1-only envelope validator. Render body_source as escaped source text and recognize const Int in fn/subdesign/design payloads; do not evaluate loops or display fabricated expanded counts. Unsupported versions must produce an explicit unsupported-format result, never an automatic downgrade to an empty summary.

Update `docs/apidocs.md` and the compiler→registry→viewer contract tests in the accepted implementation. The public version-2 form is part of the scope choice, not “automatic additive compatibility.” Sidecars remain separate from package tar/hash identity; existing best-effort docs upload must not turn a successful package publication into a failed package publication. Viewer presentation beyond an accurate signature/source view can follow separately.

No new CLI command, plugin, network call, JSON diagnostics schema or manufacturing dialect is required. Changes must not add compiler dependencies or leak unordered map iteration. M2 must not discard existing subdesign paths or port metadata in tooling. A richer exported hierarchy remains an RFC-032 implementation obligation; this proposal does not silently call that obligation complete or waive it.

## Teaching cost

Start with one and two LEDs, then an RC subdesign with named array elements and an independent layout. Explain Int versus units, one-net fan-out versus repeated nets, inclusive versus exclusive endpoints, and the boundary between type information and concrete values. The proposed label names an operation site; it is not a new circuit object. Explain B's local scope only if comparing that candidate, including why an outside layout cannot name its private locals. Source brevity and permanent conceptual cost are separate review dimensions.

## Failure modes

The acceptance table covers off-by-one, silent shorting, vanished real components, incorrectly scoped defaults, hidden empty-body errors and resource exhaustion. Also guard false claims: successful generation is not electrical simulation; computable pitch does not prove clearance; a numeric N parameter does not select a converter; an E140x block does not replace existing part/pin/DRC checks.

Keep error-bearing editor data distinct from a successful build. For example, tscircuit's [duplicate-name phase](https://github.com/tscircuit/core/blob/c298605b2779793876f20f1139a2ebd024a1d7f3/lib/components/base-components/NormalComponent/NormalComponent.ts#L236) records an error and marks the conflicting component for removal; other rendering paths can throw. CoHDL may retain a visibly failed partial editor model, but must not report a successful build after dropping a required component to recover from an error. Producing a model with error records alone proves neither a successful fabrication handoff nor nondeterministic output bytes; the relevant distinction here is the success verdict and which objects it covers.

## Migration path and coherence gate

Before acceptance, resolve A/B, label and fan-out scope choices; approve or revise the explicitly proposed uniform declaration-check correction; allocate RFC/error codes; record the decision; and update note 10 and RFC-007/024/032 plus other affected contracts together. The chosen scope must include complete subdesign indexing, ownership, ports, metering and symbolic docs, not defer them behind a fn-only implementation.

Then update the implementation plan and execute one coherent check/build/fmt/LSP/docs/Explorer delivery. Keep explicit oracles and real-part workflow evidence. Rewrite only selected future fixtures as runnable source once implemented. General electrical calculation/selection remains M3; new electrical contracts remain M4. This draft does not authorize a pure-fn migration, compiler implementation, publication or manufacturing claim.

## Decision

**Proposed — English revision ready for review; construction scope not selected.** Both candidates share typed Int/Length computation, fn/subdesign count parameters, array selectors, inherited layout behavior, uniform declaration checks and full expansion-graph budgets. Direct inst/subdesign inside a loop is Candidate B, not the recommendation. Choose the smallest coherent scope supported by the complete scenarios and an explicit cost comparison. No Accepted status, compiler implementation, executed M2 test or hardware validation is asserted.
