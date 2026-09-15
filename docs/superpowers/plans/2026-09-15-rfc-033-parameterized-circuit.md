# RFC-033 Parameterized Circuit Construction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement RFC-033 Candidate A in the CoHDL compiler: typed Int/Length compile-time expressions, local `const`, `const N: Int` generics on fn/subdesign, expression-valued array lengths/indexes/placements, mandatory-labelled half-open `for` loops over net/nc/call/placement operations, hygienic label+value iteration frames feeding the designator allocator, uniform static validation, expansion metering, the E1401–E1407 block, one-token minus lexing, fmt/LSP/docs support.

**Architecture:** One expression AST (`ast::Expr`) parsed with a small precedence-climbing parser; one evaluator (`check/eval.rs`) with two domains (Int i64 checked, Length i128 femto exact) used in three environments (definition validation with unknown values, activation, iteration); the existing expander (`check/expand.rs`) grows `Scope.consts/binders` and a labelled-frame path segment `__for_LABEL_VALUE` so every generated object rides the existing `Scope.path` → designator allocator. No text substitution, no second elaborator.

**Tech Stack:** Rust 2021, zero external crates in the compiler (hard constraint), `cargo test` fixture tests under `tests/`, mdBook unaffected.

## Global Constraints

- Zero new compiler dependencies; no HashMap iteration reaching output — BTreeMap or explicit sort everywhere output-adjacent.
- Same source + same deps → same verdict, designators, netlist bytes. Every pre-RFC fixture (`cargo test`) must stay green with identical bytes; only the enumerated §8 correction (uncalled duplicate-local / known generic mismatch now rejected) may change a verdict.
- Every diagnostic: stable code + precise span + names the exact construct. New codes: E1401 (expected Int/Length / wrong kind), E1402 (overflow / non-exact Length division), E1403 (division/remainder by zero), E1404 (reversed `for` range), E1405 (budget exceeded), E1406 (declaration/operation not admitted in this context), E1407 (const/array-length dependency cycle). Registry test `tests/error_registry.rs` enforces both directions against `docs/error-codes.md`.
- Int is a signed exact 64-bit structural integer, never a unit. Length arithmetic is exact in femto (10^-15 mm) i128; `1mm / 3` is E1402, never rounded. Computed Length text = `emit::geom::mm_femto(femto) + "mm"`; literals keep their spelling.
- Precedence: primary/parens → unary → `* / %` → `+ -` → range `..`; left-associative; source-tree evaluation order.
- `-` is always a standalone `TokenKind::Minus`; the parser assembles a signed literal only when `-` is byte-adjacent to a Number/Unit token in operand position. `10%` stays a Tolerance literal; `10 % 3` is remainder; `10%3` is a parse error.
- `for` is already a keyword; `const` and `in` are contextual identifiers (an existing trait named `Int`, or instances named `in`/`const`, keep working).
- Loop bodies (Candidate A) admit: `const`, `net`, `nc`, fn calls, nested `for`, a placement-subset `layout`. `inst`/`subdesign` inside any loop body is E1406 (E1307 wins for subdesign-in-fn). Layout loops admit `const`, `place`, nested `for` only.
- Limits: 100,000 cumulative entered iterations, 1,000,000 work items, 64 active loop frames. Charged before materialization (RFC §9 table). Metering activates only when the reachable expansion graph contains M2 syntax; a purely legacy graph keeps old behavior.
- Frame path segment: `__for_{LABEL}_{VALUE}` with negative values as `neg{N}`; `__` names stay reserved (E206). Diagnostics inside frames append ` — in <path>, <binder> = <value>` to the main message.
- No new CLI command, JSON diagnostics schema stays `schema_version: 1`; package API docs move to `schema_version: 2` only for documents containing M2 items.

---

## File structure

| File | Responsibility |
| --- | --- |
| `src/lex.rs` | `TokenKind::Minus`, `Star`, `Slash`, `DotDot`; `number()` no longer absorbs a sign |
| `src/ast.rs` | `Expr`, `BinOp`, `UnaryOp`, `ConstStmt`, `ForStmt`, `LayoutFor`, `Stmt::Const/For`, `GenericBound::Int`, `GenericDefault`, `GenericArg::Expr`; expression-valued `array_len`, `IndexSel`, `PlacementSeg.index`, `Placement.at/rotate`, `PhysAttr::Bypass.index` |
| `src/parse.rs` | `expr()` precedence parser, signed-literal assembly, `const`/`for` statements in bodies and layouts, `const N: Int` params, expression generic args |
| `src/check/eval.rs` (new) | `Value`, `Ty`, `Env`, `eval`, `type_check`, canonical Length text, E1401–E1403 |
| `src/check/generics.rs` | `GenericValue::Int`, Int params/defaults/args, E406 for device owners |
| `src/check/bodies.rs` | uniform static validation: consts, loops, E1406/E201/E1407, design bodies too |
| `src/check/expand.rs` | consts + array lengths with dependency evaluation, `.len`, expression indexes, loop frames, provenance suffix, metering |
| `src/check/meter.rs` (new) | activation walk over the syntactic expansion graph + `Meter` counters, E1405 |
| `src/resolve.rs` | `rewrite_body` recursion into `For` bodies |
| `src/fmt.rs` | expression/const/for printing; idempotence |
| `src/lsp.rs` | hover for consts/binders/`.len`; symbols inside loops |
| `src/emit/docsjson.rs` | schema v2: `bound: {"const":"Int"}`, `body_source` |
| `docs/error-codes.md`, `docs/compliance-report.md`, `docs/design/10-language-specification.md` | registry rows, ledger entry, note 10 section |
| `tests/rfc033_lex_parse.rs`, `tests/rfc033_eval.rs`, `tests/rfc033_generics.rs`, `tests/rfc033_arrays.rs`, `tests/rfc033_loops.rs`, `tests/rfc033_identity.rs`, `tests/rfc033_static.rs`, `tests/rfc033_budget.rs`, `tests/rfc033_fmt.rs`, `tests/rfc033_docs.rs` | acceptance matrix, one file per concern |

Test harness convention (copy from `tests/inst_array.rs:20-40`): `check(src)` → `(Checked, rendered)` via `cohdl::pipeline::check_files_in("board", &[("src/main.cohdl", src)], None)`; `netlist(src)` via `build_artifacts(&mut checked, &LockState::default())`. A shared `LIB` const declares synthetic devices/parts with `pub footprint FP {}` so builds are part-bound.

---

### Task 1: Lexer — one-token minus and arithmetic punctuation

**Files:**
- Modify: `src/lex.rs:42-62` (TokenKind), `:78-113` (token_text), `:192-226` (punct dispatch), `:354-441` (`number`)
- Test: `src/lex.rs` unit tests (bottom of file)

**Interfaces:**
- Produces: `TokenKind::Minus`, `TokenKind::Star`, `TokenKind::Slash`, `TokenKind::DotDot`; `Lexer::number(start)` (no `negative` argument). Unit literals are always non-negative at the lexer; the parser assembles signed literals (Task 3).

- [ ] **Step 1: Write the failing lexer tests** (append inside `mod tests` in `src/lex.rs`)

```rust
    #[test]
    fn minus_is_always_standalone() {
        let toks = lex_ok("n-1 - 1.00mm -40C");
        assert_eq!(
            toks,
            vec![
                TokenKind::Ident("n".into()),
                TokenKind::Minus,
                TokenKind::Number("1".into()),
                TokenKind::Minus,
                TokenKind::Unit(units::make_value(false, "1.00", None, UnitType::Length, "1.00mm").unwrap()),
                TokenKind::Minus,
                TokenKind::Unit(units::make_value(false, "40", None, UnitType::Temperature, "40C").unwrap()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn arithmetic_punctuation() {
        let toks = lex_ok("(n + 1) * 4mm / 2 % 3 0..N");
        assert!(toks.contains(&TokenKind::Star));
        assert!(toks.contains(&TokenKind::Slash));
        assert!(toks.contains(&TokenKind::Percent));
        assert!(toks.contains(&TokenKind::DotDot));
        // `10%` stays ONE Tolerance token; `10 % 3` is three tokens.
        let t = lex_ok("10% 10 % 3");
        assert!(matches!(t[0], TokenKind::Unit(ref v) if v.unit == UnitType::Tolerance));
        assert_eq!(t[1], TokenKind::Number("10".into()));
        assert_eq!(t[2], TokenKind::Percent);
        assert_eq!(t[3], TokenKind::Number("3".into()));
    }
```

`make_value` signature is `pub fn make_value(negative: bool, mantissa: &str, prefix: Option<SiPrefix>, unit: UnitType, full_text: &str) -> Result<UnitValue, UnitLexError>` (`src/units.rs:284`). Delete the now-obsolete `negative_voltage_rejected` test (the lexer no longer sees a sign; E105 moves to the parser in Task 3).

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --lib lex::tests -- --nocapture`
Expected: compile error `no variant named Minus` / `Star` / `Slash` / `DotDot`.

- [ ] **Step 3: Implement**

In `TokenKind` add after `Percent`:

```rust
    Minus,
    Star,
    Slash,
    /// `..` — the half-open range delimiter (`for i in 0..N`); the inclusive
    /// fan-out `..=` is still parsed as DotDot then Eq.
    DotDot,
```

In `token_text` add:

```rust
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::DotDot => "..",
```

In `run()` replace the `b'-'` arm and the `b'.'`/`b'/'` arms:

```rust
                b'/' if self.peek(1) == Some(b'/') => { /* unchanged comment skip */ }
                b'/' => self.punct(TokenKind::Slash),
                b'*' => self.punct(TokenKind::Star),
                b'-' => self.punct(TokenKind::Minus),
                b'.' => {
                    if self.peek(1) == Some(b'.') {
                        self.pos += 2;
                        self.push(TokenKind::DotDot, start);
                    } else {
                        self.punct(TokenKind::Dot);
                    }
                }
```

Change `fn number(&mut self, start: usize, negative: bool)` to `fn number(&mut self, start: usize)`; update the one call site (`b'0'..=b'9' => self.number(start)`); inside, delete the `if negative { … E102 … }` block and pass `false` to `units::make_value`. Keep everything else (Unicode traps, suffix parsing) unchanged.

Note: `..=` in `index_sel` (`src/parse.rs:3961-3975`) currently expects `Dot, Dot, Eq`; Task 3 changes it to expect `DotDot, Eq`. Until Task 3 lands, the parser tests for ranges will fail — do Task 1 and Task 3's range fix in the same commit if you want `cargo test` green at every commit; otherwise commit Task 1 with `--lib` tests only and land Task 3 immediately after.

- [ ] **Step 4: Run lexer tests**

Run: `cargo test --lib lex::tests`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/lex.rs
git commit -m "lex: one-token minus, arithmetic punctuation, half-open range token (RFC-033 §10)"
```

---

### Task 2: AST — expressions, const, for, Int generics, expression-valued positions

**Files:**
- Modify: `src/ast.rs:802-841` (generics), `:945-1000` (SubdesignUseStmt, Stmt), `:1002-1050` (LayoutBlock, Placement, PlacementSeg), `:1149-1176` (InstStmt), `:1200-1215` (PhysAttr::Bypass), `:1305-1380` (IndexSel)
- Modify (compile fixes only, literal-only behavior preserved): `src/parse.rs`, `src/fmt.rs`, `src/check/expand.rs`, `src/check/bodies.rs`, `src/check/generics.rs`, `src/lsp.rs`, `src/emit/docsjson.rs`, `src/resolve.rs`

**Interfaces:**
- Produces (used by every later task):

```rust
// src/ast.rs — new section "Expressions (RFC-033)"
#[derive(Debug, Clone)]
pub enum Expr {
    /// A decimal integer literal (already sign-combined when written `-9223372036854775808`).
    Int(i64, Span),
    /// A Length literal (`4mm`, `-1.5mm`), original text preserved in the UnitValue.
    Length(UnitValue, Span),
    /// A visible name: const, loop binder, generic parameter (Int or Length).
    Name(Ident),
    /// `ARRAY.len`
    Len(Ident, Span),
    Unary { op: UnaryOp, rhs: Box<Expr>, span: Span },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, span: Span },
    Paren(Box<Expr>, Span),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Neg, Plus }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp { Add, Sub, Mul, Div, Rem }
impl Expr {
    pub fn span(&self) -> Span { /* each variant's span */ }
    /// `Some(n)` when this is a bare integer literal (possibly parenthesized).
    pub fn as_int_literal(&self) -> Option<i64> {
        match self { Expr::Int(n, _) => Some(*n), Expr::Paren(e, _) => e.as_int_literal(), _ => None }
    }
    pub fn as_length_literal(&self) -> Option<&UnitValue> {
        match self { Expr::Length(v, _) => Some(v), Expr::Paren(e, _) => e.as_length_literal(), _ => None }
    }
    pub fn int(n: i64, span: Span) -> Expr { Expr::Int(n, span) }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstTy { Int, Length }
#[derive(Debug, Clone)]
pub struct ConstStmt { pub name: Ident, pub ty: ConstTy, pub value: Expr, pub span: Span }
#[derive(Debug, Clone)]
pub struct ForStmt {
    pub label: Ident, pub binder: Ident, pub start: Expr, pub end: Expr,
    pub body: Vec<Stmt>, pub span: Span,
}
/// A `for` inside a `layout {}` block: only const, place and nested for.
#[derive(Debug, Clone)]
pub struct LayoutFor {
    pub label: Ident, pub binder: Ident, pub start: Expr, pub end: Expr,
    pub consts: Vec<ConstStmt>, pub placements: Vec<Placement>, pub loops: Vec<LayoutFor>,
    pub span: Span,
}
// Stmt gains:  Const(ConstStmt), For(ForStmt)
// LayoutBlock gains: pub consts: Vec<ConstStmt>, pub loops: Vec<LayoutFor>
// Placement: pub at: (Expr, Expr), pub rotate: Option<Expr>   (None = 0)
// PlacementSeg: pub index: Option<(Expr, Span)>
// InstStmt / SubdesignUseStmt: pub array_len: Option<(Expr, Span)>
// PhysAttr::Bypass: index: Option<(Expr, Span)>
// IndexSel: Single(Expr, Span); Range { start: Expr, end: Expr, step: Option<Expr>, span }; List(Vec<Expr>, Span)
// GenericBound gains: Int(Span)
// GenericParam.default: Option<GenericDefault>  where
#[derive(Debug, Clone)]
pub enum GenericDefault { Unit(UnitValue, Span), Int(i64, Span) }
// GenericArg gains: Expr(Expr)   (an Int/Length expression argument)
```

`IndexSel::indices()` and `Display` become literal-only helpers: keep `Display` (print expressions via `expr_text`, Task 4 provides `ast::expr_text`), and replace `indices()` with `fn literal_indices(&self) -> Option<Vec<i64>>` returning `None` when any part is not a literal. `Placement::path_text` prints `[{}]` with `expr_text`.

- [ ] **Step 1: Add the new types and change the fields exactly as listed above.** Add `pub fn expr_text(e: &Expr) -> String` in `src/ast.rs` (canonical spelling: binary ops spaced, unary tight, parens preserved, literals by original text, `NAME.len`):

```rust
pub fn expr_text(e: &Expr) -> String {
    match e {
        Expr::Int(n, _) => n.to_string(),
        Expr::Length(v, _) => v.text.clone(),
        Expr::Name(id) => id.name.clone(),
        Expr::Len(id, _) => format!("{}.len", id.name),
        Expr::Unary { op, rhs, .. } => format!("{}{}", match op { UnaryOp::Neg => "-", UnaryOp::Plus => "+" }, expr_text(rhs)),
        Expr::Binary { op, lhs, rhs, .. } => {
            let sym = match op { BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*", BinOp::Div => "/", BinOp::Rem => "%" };
            format!("{} {} {}", expr_text(lhs), sym, expr_text(rhs))
        }
        Expr::Paren(inner, _) => format!("({})", expr_text(inner)),
    }
}
```

- [ ] **Step 2: Run `cargo build` and fix every compile error mechanically, preserving literal-only behavior.** The known sites (the compiler will list them; treat this list as the checklist):
  - `src/parse.rs`: wrap parsed numbers as `Expr::Int(n, span)` at `index_number` callers (`:3299` array length, `:3961-4020` index_sel, `:3640-3655` placement seg, `:1940-1952` bypass index); `length_pair` result into `(Expr::Length(x, sx), Expr::Length(y, sy))`; `rotate` → `Some(Expr::Int(n, span))`; `GenericParam.default` → `GenericDefault::Unit`.
  - `src/fmt.rs:1005-1009,1064-1076,1120-1125,1578-1600`: print with `expr_text`; array length `[{}; {}]` with `expr_text`; `rotate` prints `expr_text` when `Some`; `GenericDefault::Int(n, _)` prints `n`; `GenericArg::Expr(e)` prints `expr_text(e)`.
  - `src/check/expand.rs`: at every `array_len`/index/coordinate/rotate use, call `as_int_literal()`/`as_length_literal()` and, when `None`, push a temporary `E1401` "expression not supported here yet" (Task 8 replaces these with evaluation). Specifically `walk_body:251`, `handle_subdesign_use:1979`, `indexed_local:372`, `handle_placement:440-470,560-612`, `array_bounds:1223` (use `literal_indices()`), `resolve_array_ref:1292`, `expand_member:1258`, bypass handling in `handle_inst_phys`.
  - `src/check/bodies.rs`, `src/check/generics.rs`: match the new `GenericBound::Int` / `GenericDefault` / `GenericArg::Expr` arms with the same temporary E1401 (Task 7 implements them); add `Stmt::Const(_) | Stmt::For(_) => {}` arms (Task 10 implements).
  - `src/resolve.rs:667`: add `Stmt::Const(_) => {}` and `Stmt::For(f) => self.rewrite_body(&mut f.body, module, shadow, diags)` (recursion is required now, or nested loop bodies keep unresolved short names).
  - `src/lsp.rs:1044,1954`, `src/emit/docsjson.rs:226`: add `Stmt::Const(_) | Stmt::For(_) => {}` arms; docsjson `array` field prints `expr_text`; generics `bound` for `Int` → `Val::Obj(vec![("const", s("Int"))])` (Task 14 finalizes schema bump).

- [ ] **Step 3: Run the whole suite to prove byte-stability**

Run: `cargo test`
Expected: all existing tests pass (every fixture is literal-only, so `as_int_literal` paths cover them). `cargo run -- fmt examples lib --check` also reports canonical form.

- [ ] **Step 4: Commit**

```bash
git add src/ast.rs src/parse.rs src/fmt.rs src/check src/resolve.rs src/lsp.rs src/emit/docsjson.rs
git commit -m "ast: expression nodes, const/for statements, Int generics; literal-only plumbing (RFC-033 §2-§5)"
```

---

### Task 3: Parser — expression grammar, signed literals, `const`, `for`, Int generics

**Files:**
- Modify: `src/parse.rs:2477-2600` (generic params/args), `:3236-3260` (stmt dispatch), `:3552-3600` (layout block), `:3630-3700` (placement), `:3929-4020` (index_number/index_sel), `:1166-1200` (length_pair)
- Test: `tests/rfc033_lex_parse.rs` (new)

**Interfaces:**
- Produces: `Parser::expr(&mut self) -> Option<Expr>`; `Parser::const_stmt()`, `Parser::for_stmt()`, `Parser::layout_for()`; `index_sel` now yields expression selectors; `signed_literal_or_number()` for legacy consumers (pin numbers, pad numbers) that must keep E102/E105.

Grammar (precedence climbing, no backtracking, one-token lookahead):

```text
expr    := add
add     := mul (('+' | '-') mul)*
mul     := unary (('*' | '/' | '%') unary)*
unary   := ('-' | '+') unary | primary
primary := Number | Unit(Length) | '(' expr ')' | Ident '.' 'len' | Ident
```

Signed-literal assembly: in `unary`, when the token is `Minus` and the NEXT token is `Number`/`Unit` and `next.span.start == minus.span.end` (byte-adjacent), consume both and produce `Expr::Int(-n)` (range-checked so `-9223372036854775808` works: parse the magnitude as `u64`, reject > 2^63) or `Expr::Length(negated UnitValue with text "-…")`. For a non-Length unit (`-5V`) push E105 exactly as the lexer used to. A space/comment/paren between `-` and the literal falls through to `Unary { Neg }`.

- [ ] **Step 1: Write failing parse tests** (`tests/rfc033_lex_parse.rs`)

```rust
use cohdl::pipeline::check_files_in;

fn parse_render(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut c = check_files_in("board", &files, None).expect("selection");
    c.diags.sort(&c.sm);
    c.diags.render(&c.sm)
}

const LIB: &str = r#"
pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
"#;

#[test]
fn const_and_for_parse() {
    let src = format!("{LIB}
design Board {{
    const N: Int = 2 + 1
    const PITCH: Length = 4mm
    inst d: [Dev; N]
    for links: n in 0..(N - 1) {{
        net _: d[n].B, d[n + 1].A
    }}
    net IN: d[0].A
    nc: d[N - 1].B
    layout {{
        for grid: n in 0..d.len {{
            place d[n] at (10mm + n * PITCH, -1.5mm)
        }}
    }}
}}");
    let r = parse_render(&src);
    assert!(!r.contains("E010"), "no parse errors expected:\n{r}");
}

#[test]
fn minus_forms() {
    // n-1 and n - 1 are subtraction; -1.00mm is a literal; - 1.00mm is unary.
    let src = format!("{LIB}
design Board {{
    const A: Int = 3
    const B: Int = A-1
    const C: Int = A - 1
    const D: Length = -1.00mm
    const E: Length = - 1.00mm
    const F: Int = -9223372036854775808
    inst d: Dev
    net _: d.A, d.B
}}");
    let r = parse_render(&src);
    assert!(!r.contains("E010") && !r.contains("E001"), "{r}");
}

#[test]
fn negative_voltage_still_e105_and_tolerance_lexing() {
    let src = format!("{LIB}
pub device V<X: Voltage = -5V> {{ pins {{ A: 1 [passive] }} spec {{ v: X }} }}
design Board {{ inst d: Dev  net _: d.A, d.B }}");
    assert!(parse_render(&src).contains("E105"));
    let src2 = format!("{LIB}
design Board {{ const T: Int = 10%3  inst d: Dev  net _: d.A, d.B }}");
    assert!(parse_render(&src2).contains("E010"));
}

#[test]
fn int_generic_param_and_expr_args() {
    let src = format!("{LIB}
fn bank<const N: Int = 2, L: Length>(p: Pin) {{ net _: p }}
design Board {{ inst d: Dev  bank::<1 + 1, 2mm * 2>(d.A)  net _: d.B }}");
    let r = parse_render(&src);
    assert!(!r.contains("E010"), "{r}");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --test rfc033_lex_parse`
Expected: FAIL with E010 diagnostics (grammar not implemented).

- [ ] **Step 3: Implement the expression parser** (add near `index_number`):

```rust
    fn expr(&mut self) -> Option<Expr> { self.expr_add() }

    fn expr_add(&mut self) -> Option<Expr> {
        let mut lhs = self.expr_mul()?;
        loop {
            let op = match self.peek() { TokenKind::Plus => BinOp::Add, TokenKind::Minus => BinOp::Sub, _ => break };
            self.bump();
            let rhs = self.expr_mul()?;
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
        }
        Some(lhs)
    }

    fn expr_mul(&mut self) -> Option<Expr> {
        let mut lhs = self.expr_unary()?;
        loop {
            let op = match self.peek() { TokenKind::Star => BinOp::Mul, TokenKind::Slash => BinOp::Div, TokenKind::Percent => BinOp::Rem, _ => break };
            self.bump();
            let rhs = self.expr_unary()?;
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
        }
        Some(lhs)
    }

    fn expr_unary(&mut self) -> Option<Expr> {
        let start = self.span();
        match self.peek() {
            TokenKind::Minus => {
                let minus = self.bump();
                // Signed-literal assembly: byte-adjacent number/unit.
                let adjacent = self.span().start == minus.span.end;
                match self.peek() {
                    TokenKind::Number(_) if adjacent => {
                        let t = self.bump();
                        let TokenKind::Number(text) = t.kind else { unreachable!() };
                        let span = minus.span.to(t.span);
                        return match text.parse::<u64>() {
                            Ok(m) if m <= (1u64 << 63) => Some(Expr::Int((m as i128).wrapping_neg() as i64, span)),
                            _ => { self.diags.push(Diagnostic::error("E1402", span, format!("`-{}` is out of range for an Int (−2^63 … 2^63−1)", text))); None }
                        };
                    }
                    TokenKind::Unit(_) if adjacent => {
                        let t = self.bump();
                        let TokenKind::Unit(v) = t.kind else { unreachable!() };
                        let span = minus.span.to(t.span);
                        if !v.unit.allows_negative() {
                            self.diags.push(Diagnostic::error("E105", span, format!("`{}` cannot be negative — only `Temperature` and `Length` literals may carry a leading `-`", v.unit.type_name())));
                            return None;
                        }
                        let neg = UnitValue { unit: v.unit, femto: -v.femto, text: format!("-{}", v.text) };
                        return Some(Expr::Length(neg, span));  // Temperature literals never reach expr(); see signed_unit_literal
                    }
                    _ => {}
                }
                let rhs = self.expr_unary()?;
                let span = start.to(rhs.span());
                Some(Expr::Unary { op: UnaryOp::Neg, rhs: Box::new(rhs), span })
            }
            TokenKind::Plus => { self.bump(); let rhs = self.expr_unary()?; let span = start.to(rhs.span()); Some(Expr::Unary { op: UnaryOp::Plus, rhs: Box::new(rhs), span }) }
            _ => self.expr_primary(),
        }
    }

    fn expr_primary(&mut self) -> Option<Expr> {
        match self.peek() {
            TokenKind::Number(_) => {
                let t = self.bump();
                let TokenKind::Number(text) = t.kind else { unreachable!() };
                match text.parse::<i64>() {
                    Ok(n) => Some(Expr::Int(n, t.span)),
                    Err(_) => { self.diags.push(Diagnostic::error("E1401", t.span, format!("`{}` is not an Int — integer literals are whole decimal numbers in −2^63 … 2^63−1", text))); None }
                }
            }
            TokenKind::Unit(_) => {
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else { unreachable!() };
                if v.unit == UnitType::Length { Some(Expr::Length(v, t.span)) } else {
                    self.diags.push(Diagnostic::error("E1401", t.span, format!("expected an Int or Length expression, found `{}` (a `{}`)", v.text, v.unit.type_name())));
                    None
                }
            }
            TokenKind::LParen => {
                let open = self.span(); self.bump();
                let inner = self.expr()?;
                self.expect(&TokenKind::RParen, "to close the parenthesized expression");
                Some(Expr::Paren(Box::new(inner), open.to(self.prev_span())))
            }
            TokenKind::Ident(_) => {
                let id = self.ident("in an expression")?;
                if self.at(&TokenKind::Dot) && matches!(self.peek_ahead(1), TokenKind::Ident(n) if n == "len") {
                    self.bump(); let t = self.bump();
                    return Some(Expr::Len(id.clone(), id.span.to(t.span)));
                }
                Some(Expr::Name(id))
            }
            other => { let msg = format!("expected an Int or Length expression, found {}", other.describe()); self.error_here(msg); None }
        }
    }
```

Legacy consumers keep their codes: add `fn legacy_number(&mut self, ctx: &str) -> Option<i64>` that, on `Minus` followed by adjacent `Number`, emits **E102** ("a bare number cannot be negative…") and returns None; use it for pin numbers, pad numbers, mount-hole numbers. Add `fn signed_unit_literal(&mut self, ctx)` (Minus + adjacent Unit → negated value for Temperature/Length, E105 otherwise) and use it wherever `unit_literal` accepted `-40C`/`-1.5mm` before (spec values, generic defaults, pad/footprint coordinates, silkscreen geometry — grep `unit_literal(` and `length_pair(`).

- [ ] **Step 4: Statements.** In `stmt()` before the `layout` check:

```rust
        if matches!(self.peek(), TokenKind::Ident(n) if n == "const")
            && matches!(self.peek_ahead(1), TokenKind::Ident(_)) && self.peek_ahead(2) == &TokenKind::Colon {
            self.reject_attrs(&attrs); self.reject_phys(&phys, "a `const`");
            return self.const_stmt().map(Stmt::Const);
        }
        if self.at(&TokenKind::For) {
            self.reject_attrs(&attrs); self.reject_phys(&phys, "a `for` loop");
            return self.for_stmt().map(Stmt::For);
        }
```

```rust
    fn const_stmt(&mut self) -> Option<ConstStmt> {
        let start = self.span(); self.bump(); // const
        let name = self.ident("as the constant name")?;
        self.expect(&TokenKind::Colon, "after the constant name");
        let ty_id = self.ident("as the constant type (`Int` or `Length`)")?;
        let ty = match ty_id.name.as_str() { "Int" => ConstTy::Int, "Length" => ConstTy::Length, other => {
            self.diags.push(Diagnostic::error("E1401", ty_id.span, format!("`{}` is not a constant type — a `const` is `Int` or `Length`", other))); return None; } };
        self.expect(&TokenKind::Eq, "before the constant's value");
        let value = self.expr()?;
        Some(ConstStmt { name, ty, value, span: start.to(self.prev_span()) })
    }

    fn for_stmt(&mut self) -> Option<ForStmt> {
        let (label, binder, start_e, end_e, start) = self.for_header()?;
        self.expect(&TokenKind::LBrace, "to open the loop body");
        let mut body = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            if let Some(s) = self.stmt() { body.push(s); }
            if self.pos == before { self.bump(); }
        }
        self.expect(&TokenKind::RBrace, "to close the loop body");
        Some(ForStmt { label, binder, start: start_e, end: end_e, body, span: start.to(self.prev_span()) })
    }

    /// `for LABEL: IDENT in expr .. expr` — shared by body and layout loops.
    fn for_header(&mut self) -> Option<(Ident, Ident, Expr, Expr, Span)> {
        let start = self.span(); self.bump(); // for
        let label = self.ident("as the loop label (every loop is labelled: `for links: n in 0..N`)")?;
        self.expect(&TokenKind::Colon, "after the loop label");
        let binder = self.ident("as the loop variable")?;
        match self.peek() { TokenKind::Ident(n) if n == "in" => { self.bump(); } other => { let m = format!("expected `in` after the loop variable, found {}", other.describe()); self.error_here(m); return None; } }
        let lo = self.expr()?;
        if !self.expect(&TokenKind::DotDot, "as the half-open range delimiter `..` (loops are exclusive at the end; `..=` is only for net fan-out)") { return None; }
        let hi = self.expr()?;
        Some((label, binder, lo, hi, start))
    }
```

Layout block: in `layout_block()` add arms `else if self.at_ident("const") { consts.push(self.const_stmt()?) } else if self.at(&TokenKind::For) { loops.push(self.layout_for()?) }` where `layout_for` uses `for_header` then loops over `const`/`place`/`for` only, pushing `E1406` for `inst`/`net`/`nc`/calls ("`{kw}` is not admitted inside a layout loop — only `const`, `place` and nested `for`"). `placement()`: `at` via `self.expr()` twice; `rotate` via `self.expr()`; segment index via `self.expr()` inside `[...]` (a `..`/`,` after it is E211 "place takes a single element"). `index_sel()`: first = `self.expr()`; range on `DotDot` then expect `Eq`; `step` = `self.expr()`; list entries `self.expr()`. Array length in `inst`/`subdesign`: `self.expr()`. Generic params: after the name, if the next token is `Ident("const")`… note `const` precedes the name: `<const N: Int = 2>` — check `at_ident("const")` at parameter start, then name, `:`, expect `Ident("Int")` (E406 otherwise: "`const` parameters are `Int`"), default via `legacy_number`-style integer literal only. Generic args: `Number`/`Unit(Length)`/`(`/`-` start an `Expr` → `GenericArg::Expr(self.expr()?)`; a non-Length unit literal stays `GenericArg::Unit`; a bare `Ident` stays `GenericArg::Name` unless followed by an operator or `.len`, in which case parse `expr()`.

- [ ] **Step 5: Run tests**

Run: `cargo test --test rfc033_lex_parse && cargo test`
Expected: new tests pass; full suite green (fmt round-trips are literal-only until Task 4; if `fmt --check` on examples fails because `rotate`/index printing changed, fix `expr_text` usage first).

- [ ] **Step 6: Commit**

```bash
git add src/parse.rs tests/rfc033_lex_parse.rs
git commit -m "parse: expression grammar, const/for statements, const Int generics (RFC-033 §3-§5, §10)"
```

---

### Task 4: `cohdl fmt` — canonical expression, const and for formatting

**Files:**
- Modify: `src/fmt.rs:962-1140` (stmt), `:1031-1110` (layout), `:1578-1610` (generic text)
- Test: `tests/rfc033_fmt.rs` (new)

**Interfaces:**
- Consumes: `ast::expr_text`, `Stmt::Const/For`, `LayoutBlock.consts/loops`.
- Produces: canonical form — `const NAME: Ty = <expr>`; `for LABEL: i in a..b {` … `}` with the body indented one level, printed in source order among sibling statements; layout loops likewise; binary operators single-spaced, unary tight, parentheses preserved exactly as written.

- [ ] **Step 1: Write failing tests** (`tests/rfc033_fmt.rs`)

```rust
use cohdl::fmt::format_source;

const SRC: &str = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    const N: Int = (2+1)*2
    inst d: [Dev; N]
    for links: n in 0..N-1 {
        net _: d[n].B, d[n+1].A
    }
    net IN: d[0].A
    nc: d[N - 1].B
    layout {
        for grid: n in 0..d.len {
            place d[n] at (10mm+n*4mm, -1.5mm) rotate 90*n
        }
    }
}
";

#[test]
fn canonical_and_idempotent() {
    let once = format_source("main.cohdl", SRC).unwrap();
    assert!(once.contains("const N: Int = (2 + 1) * 2"));
    assert!(once.contains("for links: n in 0..N - 1 {"));
    assert!(once.contains("net _: d[n].B, d[n + 1].A"));
    assert!(once.contains("place d[n] at (10mm + n * 4mm, -1.5mm) rotate 90 * n"));
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn parentheses_are_never_dropped() {
    let src = "design Board { const A: Int = (1 + 2) * 3 }\n";
    let out = format_source("main.cohdl", src).unwrap();
    assert!(out.contains("(1 + 2) * 3"));
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test --test rfc033_fmt` fails (const/for not printed, parens/spacing wrong).

- [ ] **Step 3: Implement.** In `stmt()` add:

```rust
            Stmt::Const(c) => {
                self.flush_leading(self.line_start(c.span), indent);
                let ty = match c.ty { ConstTy::Int => "Int", ConstTy::Length => "Length" };
                self.push(indent, format!("const {}: {} = {}", c.name.name, ty, expr_text(&c.value)));
            }
            Stmt::For(f) => {
                let held = self.hold_line_comment(self.line_start(f.span), self.line_end(f.span));
                self.flush_leading(self.line_start(f.span), indent);
                self.push(indent, format!("for {}: {} in {}..{} {{", f.label.name, f.binder.name, expr_text(&f.start), expr_text(&f.end)));
                self.attach_trailing(self.line_start(f.span));
                for s in &f.body { self.stmt(s, indent + 1); }
                self.flush_leading(self.line_end(f.span), indent + 1);
                self.push(indent, "}");
                self.append_held(held);
                self.cursor = self.cursor.max(self.line_end(f.span) + 1);
            }
```

Layout: print `consts` then `constraints` (existing order), then `board_outline`, then `placements`, then `loops` via a recursive `fn layout_for(&mut self, f: &LayoutFor, indent)` that prints header, consts, placements, nested loops, `}`. Placement line: `format!("place {} at ({}, {}){}{}", p.path_text(), expr_text(&p.at.0), expr_text(&p.at.1), rot, side)` where `rot` is `format!(" rotate {}", expr_text(e))` for `Some(e)` unless `e.as_int_literal() == Some(0)`. `generic_params`: `GenericBound::Int(_)` → `const {name}: Int`, default `GenericDefault::Int(n,_)` → ` = {n}`. `generic_arg_text`: `GenericArg::Expr(e)` → `expr_text(e)`.

- [ ] **Step 4: Run** `cargo test --test rfc033_fmt && cargo run -- fmt examples lib book/examples --check` — all canonical.

- [ ] **Step 5: Commit** `git add src/fmt.rs tests/rfc033_fmt.rs && git commit -m "fmt: canonical expressions, const and for (RFC-033 tooling)"`

---

### Task 5: Evaluator — `src/check/eval.rs`

**Files:**
- Create: `src/check/eval.rs`; register `pub mod eval;` in `src/check/mod.rs`
- Test: unit tests inside `src/check/eval.rs` + `tests/rfc033_eval.rs` (end-to-end via `const` in a design)

**Interfaces:**
- Produces:

```rust
pub enum Value { Int(i64), Length(i128) }            // Length in femto-mm
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ty { Int, Length }
/// What a name means during evaluation/type-checking.
pub enum NameKind { Const(Value), Binder(i64), GenericInt(i64), GenericLength(UnitValue), Unknown(Ty) }
pub struct Env<'a> {
    pub names: &'a BTreeMap<String, NameKind>,   // consts, binders, generic params (Int/Length only)
    pub array_lens: &'a BTreeMap<String, i64>,   // visible physical + subdesign arrays with KNOWN length
    pub unknown_arrays: &'a BTreeSet<String>,    // arrays whose length is not yet concrete (definition validation)
}
/// Type-check with possibly-unknown values. Returns the type or None after pushing E1401.
pub fn type_check(e: &Expr, env: &Env, diags: &mut Diagnostics) -> Option<Ty>;
/// Evaluate fully. Returns None after pushing E1401/E1402/E1403 (or E202 for unknown names).
pub fn eval(e: &Expr, env: &Env, diags: &mut Diagnostics) -> Option<Value>;
/// `Value::Length` → canonical text `"<mm_femto>mm"`; a literal keeps its own text (handled by callers).
pub fn length_text(femto: i128) -> String { format!("{}mm", crate::emit::geom::mm_femto(femto)) }
/// Length value → UnitValue with canonical text (for IR placements/generic args).
pub fn length_value(femto: i128) -> UnitValue;
/// A frame-provenance suffix appended by callers to E202/E1402/E1403 main messages; empty at top level.
```

Rules implemented exactly per RFC §2: Int `+ - *` checked (`checked_add/sub/mul` → E1402 "Int overflow in `<expr_text>`"), `/` and `%` truncate toward zero, zero divisor E1403 ("division by zero in `<expr_text>`"), `i64::MIN / -1` and `% -1` E1402; Length `+ -` in i128 checked (i128 overflow → E1402); `Int * Length` / `Length * Int` checked; `Length / Int` exact only (`femto % n != 0` → E1402 "`<expr>` is not exactly representable — Length division must be exact"); `Int + Length`, `Length / Length`, `Length * Length`, unary on unknown-typed names → E1401 with the two operand types named ("`<lhs>` is an Int and `<rhs>` is a Length — Int and Length cannot be added"). Unary `-` on `i64::MIN` value → E1402. `.len` on an unknown array (definition validation) → `Ty::Int` without a value; on an unknown NAME → E202 "unknown array `X` — `.len` reads a visible array's declared length". `type_check` evaluates constant subexpressions where both operands are concrete (so `i / 0` inside an empty loop still reports E1403 — a known zero divisor is an invariant failure), and never invents values for `Unknown` names.

- [ ] **Step 1: Write failing unit tests** (in `src/check/eval.rs` `mod tests`; build `Expr` trees by parsing snippets through a tiny helper that runs `crate::parse::parse` on `design X { const A: Int = <expr> }` and pulls the `ConstStmt` value):

```rust
    fn e(src: &str) -> Expr { /* parse `design D { const A: Int = {src} }`, return body[0] const value */ }
    fn env_empty() -> (BTreeMap<String, NameKind>, BTreeMap<String, i64>, BTreeSet<String>) { Default::default() }
    fn ev(src: &str) -> Result<Value, String> { /* eval with empty env; Err(rendered diags) */ }

    #[test] fn precedence_and_assoc() {
        assert!(matches!(ev("2 + 3 * 4"), Ok(Value::Int(14))));
        assert!(matches!(ev("10 - 4 - 3"), Ok(Value::Int(3))));
        assert!(matches!(ev("7 / 2"), Ok(Value::Int(3))));
        assert!(matches!(ev("-7 / 2"), Ok(Value::Int(-3))));
        assert!(matches!(ev("-7 % 2"), Ok(Value::Int(-1))));
        assert!(matches!(ev("(1 + 2) * 3"), Ok(Value::Int(9))));
    }
    #[test] fn int_domain_errors() {
        assert!(ev("9223372036854775807 + 1").unwrap_err().contains("E1402"));
        assert!(ev("-9223372036854775808 / -1").unwrap_err().contains("E1402"));
        assert!(ev("1 / 0").unwrap_err().contains("E1403"));
        assert!(ev("1 % 0").unwrap_err().contains("E1403"));
        assert!(ev("-(-9223372036854775808)").unwrap_err().contains("E1402"));
    }
    #[test] fn length_domain() {
        assert!(matches!(ev("10mm + 2 * 4mm"), Ok(Value::Length(18_000_000_000_000_000))));
        assert!(matches!(ev("1.00mm + 0mm"), Ok(Value::Length(1_000_000_000_000_000))));
        assert_eq!(length_text(1_000_000_000_000_000), "1mm");
        assert_eq!(length_text(-500_000_000_000_000), "-0.5mm");
        assert!(matches!(ev("1mm / 8"), Ok(Value::Length(125_000_000_000_000))));
        assert!(ev("1mm / 3").unwrap_err().contains("E1402"));
        assert!(ev("10mm + 2").unwrap_err().contains("E1401"));
        assert!(ev("1mm * 1mm").unwrap_err().contains("E1401"));
        assert!(ev("1mm / 0").unwrap_err().contains("E1403"));
    }
```

- [ ] **Step 2: Run** `cargo test --lib check::eval` — fails (module missing).

- [ ] **Step 3: Implement `eval.rs`.** Core:

```rust
pub fn eval(e: &Expr, env: &Env, diags: &mut Diagnostics) -> Option<Value> {
    match e {
        Expr::Int(n, _) => Some(Value::Int(*n)),
        Expr::Length(v, _) => Some(Value::Length(v.femto)),
        Expr::Paren(inner, _) => eval(inner, env, diags),
        Expr::Name(id) => match env.names.get(&id.name) {
            Some(NameKind::Const(v)) => Some(v.clone()),
            Some(NameKind::Binder(i)) | Some(NameKind::GenericInt(i)) => Some(Value::Int(*i)),
            Some(NameKind::GenericLength(u)) => Some(Value::Length(u.femto)),
            Some(NameKind::Unknown(_)) => None, // caller decides (definition validation never calls eval on unknowns)
            None => { diags.push(Diagnostic::error("E202", id.span, format!("unknown name `{}` in this expression — expected a const, a loop variable or an Int/Length generic parameter", id.name))); None }
        },
        Expr::Len(id, span) => match env.array_lens.get(&id.name) {
            Some(n) => Some(Value::Int(*n)),
            None => { diags.push(Diagnostic::error("E202", *span, format!("unknown array `{}` — `.len` reads a visible array's declared length", id.name))); None }
        },
        Expr::Unary { op, rhs, span } => {
            let v = eval(rhs, env, diags)?;
            match (op, v) {
                (UnaryOp::Plus, v) => Some(v),
                (UnaryOp::Neg, Value::Int(i)) => i.checked_neg().map(Value::Int).or_else(|| overflow(diags, *span, e)),
                (UnaryOp::Neg, Value::Length(f)) => f.checked_neg().map(Value::Length).or_else(|| overflow(diags, *span, e)),
            }
        }
        Expr::Binary { op, lhs, rhs, span } => {
            let l = eval(lhs, env, diags)?; let r = eval(rhs, env, diags)?;
            binary(*op, l, r, *span, e, diags)
        }
    }
}

fn binary(op: BinOp, l: Value, r: Value, span: Span, e: &Expr, diags: &mut Diagnostics) -> Option<Value> {
    use Value::*;
    match (op, l, r) {
        (BinOp::Add, Int(a), Int(b)) => a.checked_add(b).map(Int).or_else(|| overflow(diags, span, e)),
        (BinOp::Sub, Int(a), Int(b)) => a.checked_sub(b).map(Int).or_else(|| overflow(diags, span, e)),
        (BinOp::Mul, Int(a), Int(b)) => a.checked_mul(b).map(Int).or_else(|| overflow(diags, span, e)),
        (BinOp::Div, Int(_), Int(0)) | (BinOp::Rem, Int(_), Int(0)) => div_zero(diags, span, e),
        (BinOp::Div, Int(a), Int(b)) => a.checked_div(b).map(Int).or_else(|| overflow(diags, span, e)),
        (BinOp::Rem, Int(a), Int(b)) => a.checked_rem(b).map(Int).or_else(|| overflow(diags, span, e)),
        (BinOp::Add, Length(a), Length(b)) => a.checked_add(b).map(Length).or_else(|| overflow(diags, span, e)),
        (BinOp::Sub, Length(a), Length(b)) => a.checked_sub(b).map(Length).or_else(|| overflow(diags, span, e)),
        (BinOp::Mul, Int(a), Length(b)) | (BinOp::Mul, Length(b), Int(a)) => (b).checked_mul(a as i128).map(Length).or_else(|| overflow(diags, span, e)),
        (BinOp::Div, Length(_), Int(0)) => div_zero(diags, span, e),
        (BinOp::Div, Length(a), Int(b)) => {
            let b = b as i128;
            if a % b != 0 { diags.push(Diagnostic::error("E1402", span, format!("`{}` is not exactly representable — Length division must be exact (no rounding)", expr_text(e)))); return None; }
            Some(Length(a / b))
        }
        (op, l, r) => { diags.push(Diagnostic::error("E1401", span, format!("`{}`: {} {} {} is not a supported operation — Int combines with Int; Length adds/subtracts Length, scales by Int, divides by Int", expr_text(e), ty_name(&l), op_sym(op), ty_name(&r)))); None }
    }
}
```

`type_check` mirrors `binary` on `Ty` values (`Int op Int → Int`, `Length ± Length → Length`, `Int * Length → Length`, `Length / Int → Length`, else E1401) and, when both operands are concrete literals/consts, also runs `binary` to surface invariant E1402/E1403.

- [ ] **Step 4: Run** `cargo test --lib check::eval` — pass.
- [ ] **Step 5: Commit** `git add src/check/eval.rs src/check/mod.rs && git commit -m "check: Int/Length expression evaluator with exact Length arithmetic (RFC-033 §2)"`

---

### Task 6: Generics — `const N: Int` on fn and subdesign

**Files:**
- Modify: `src/check/generics.rs:14-25` (GenericValue), `:30-84` (resolve_generic_args), `:85-104` (describe_param), `:121-260` (resolve_one)
- Modify: `src/check/mod.rs:25` (`check_parts` neighbourhood — add E406 owner check for devices), `src/check/bodies.rs:68-86` (int generics set)
- Test: `tests/rfc033_generics.rs` (new)

**Interfaces:**
- Consumes: `GenericBound::Int`, `GenericDefault`, `GenericArg::Expr`, `eval::{eval, Env, NameKind, Value}`.
- Produces: `GenericValue::Int(i64)`; `Substitution` may now carry Int values; helper `pub fn subst_names(subst: &Substitution) -> BTreeMap<String, NameKind>` (Int → `GenericInt`, Length unit → `GenericLength`, other units/devices omitted) used by every `Env` construction in the expander.

- [ ] **Step 1: Write failing tests** (`tests/rfc033_generics.rs`, harness as in `tests/inst_array.rs`)

```rust
const LIB: &str = r#"
pub trait Cap { designator_prefix: "C" }
pub device CapDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Cap for CapDev {}
pub footprint FP {}
pub part C100N: CapDev { primary { mfr: "m", mpn: "c", footprint: FP } }
pub device Host { pins { P: 1 [passive], Q: 2 [passive] } }
pub part HOST: Host { primary { mfr: "m", mpn: "h", footprint: FP } }
"#;

#[test]
fn fn_const_int_param_binds_and_forwards() {
    let src = format!("{LIB}
pub fn inner<const N: Int>(p: Pin) {{ inst c: [C100N; N]  net _: p, c[0..=(N - 1)].A  nc: c[0..=(N - 1)].B }}
pub fn outer<const N: Int = 2>(p: Pin) {{ inner::<N + 1>(p) }}
design Board {{ inst h: HOST  outer::<>(h.P)  nc: h.Q }}");
    let (c, r) = check(&src);
    assert!(!c.diags.has_errors(), "{r}");
    let n = netlist(&src);
    assert_eq!(n.matches("(ref C").count(), 3, "N=2 default → inner gets 3");
}

#[test]
fn subdesign_const_int_param() {
    let src = format!("{LIB}
pub subdesign Bank<const N: Int> {{ ports {{ required IN: Pin }}  inst c: [C100N; N]  net _: IN, c[0..=(N - 1)].A  nc: c[0..=(N - 1)].B }}
design Board {{ inst h: HOST  subdesign b: Bank<4> {{ IN: h.P }}  nc: h.Q }}");
    let n = netlist(&src);
    assert_eq!(n.matches("(ref C").count(), 4);
}

#[test]
fn kind_mismatches() {
    let src = format!("{LIB}
pub fn f<const N: Int>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  f::<100nF>(h.P)  nc: h.Q }}");
    assert!(check(&src).1.contains("E1401"), "unit literal for an Int parameter");
    let src = format!("{LIB}
pub fn g<V: Voltage>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  g::<3>(h.P)  nc: h.Q }}");
    assert!(check(&src).1.contains("E113"), "bare number for a unit parameter stays E113");
    let src = format!("{LIB}
pub device D<const N: Int> {{ pins {{ A: 1 [passive] }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E406"), "const generics on devices are rejected");
    let src = format!("{LIB}
pub fn f<const N: Int = 2mm>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E406"), "Int default must be an integer literal");
}
```

Note `outer::<>(…)`: if the parser rejects an empty turbofish, write `outer(h.P)` instead — RFC keeps the existing call forms.

- [ ] **Step 2: Run** `cargo test --test rfc033_generics` — fails.

- [ ] **Step 3: Implement.** `GenericValue::Int(i64)`. In `resolve_generic_args`, defaults: `GenericDefault::Unit(v,_) → Unit(v)`, `GenericDefault::Int(n,_) → Int(n)`. In `resolve_one` add arms:

```rust
        (GenericBound::Int(_), GenericArg::Expr(e)) | (GenericBound::Int(_), GenericArg::Number(..)) => {
            let e = match arg { GenericArg::Expr(e) => e.clone(), GenericArg::Number(n, sp) => Expr::Int(n.parse().unwrap_or(0), *sp), _ => unreachable!() };
            let names = subst_names(env); let empty_lens = BTreeMap::new(); let empty_unknown = BTreeSet::new();
            match eval(&e, &Env { names: &names, array_lens: &empty_lens, unknown_arrays: &empty_unknown }, diags)? {
                Value::Int(i) => Some(GenericValue::Int(i)),
                Value::Length(_) => { diags.push(Diagnostic::error("E1401", e.span(), format!("`{}` is a Length, but `const {}: Int` expects an Int", expr_text(&e), param.name.name))); None }
            }
        }
        (GenericBound::Int(_), GenericArg::Unit(v, span)) => { diags.push(Diagnostic::error("E1401", *span, format!("`{}` is a `{}`, but `const {}: Int` expects an Int (a count, not a unit value)", v.text, v.unit.type_name(), param.name.name))); None }
        (GenericBound::Int(_), GenericArg::Name(name)) => match env.get(&name.name) {
            Some(GenericValue::Int(i)) => Some(GenericValue::Int(*i)),
            Some(other) => { diags.push(Diagnostic::error("E1401", name.span, format!("`{}` is not an Int here, but `const {}: Int` expects one", name.name, param.name.name))); None }
            None => { diags.push(Diagnostic::error("E405", name.span, format!("`{}` is not a generic parameter in scope here", name.name))); None }
        },
        (GenericBound::Unit(u), GenericArg::Expr(e)) if u.unit == UnitType::Length => { /* eval; Length → GenericValue::Unit(length_value(f)); Int → E1401 */ }
        (GenericBound::Unit(u), GenericArg::Expr(e)) => { E1401: "`<expr>` is an expression — only Int and Length arguments may be computed; `<param>` expects a `<unit>` literal" }
        (GenericBound::Traits(_), GenericArg::Expr(e)) => { E403: "expects a device type, found an expression" }
```

Existing `(GenericBound::Unit(_), GenericArg::Name)` arm: `Some(GenericValue::Int(_))` → E112 "resolves to an Int count, but `X` expects a `Voltage`". Existing `(Traits, Name)` arm: `Some(GenericValue::Int(_))` → E403. `describe_param`: `GenericBound::Int(_) => "`N` expects an Int count (e.g. `4`)"`. E406 owner check: in `check_declarations` (`src/check/mod.rs`) iterate `world.devices` and push E406 on any `GenericBound::Int` param ("integer generics are not admitted on `device` declarations — pin interfaces are structural variants (RFC-008)"); in the parser, `const N: Int = <non-integer>` is E406 ("an Int default must be an integer literal"). `bodies.rs check_one`: collect `int_generics`; an `inst x: N` where N is an Int generic → E205 (existing message path).

- [ ] **Step 4: Run** `cargo test --test rfc033_generics && cargo test` — green.
- [ ] **Step 5: Commit** `git add src/check tests/rfc033_generics.rs && git commit -m "generics: const Int parameters, defaults and expression arguments for fn/subdesign (RFC-033 §3)"`

---

### Task 7: Expander — constants, computed array lengths, `.len`, expression selectors and placements

**Files:**
- Modify: `src/check/expand.rs:88-103` (Scope), `:241-313` (walk_body), `:372-427` (indexed_local), `:427-612` (handle_placement), `:963-1000` (handle_inst), `:1223-1344` (array_bounds/expand_member/resolve_array_ref), `:1979-1985` (subdesign array_len), bypass index in `handle_inst_phys`
- Test: `tests/rfc033_arrays.rs` (new)

**Interfaces:**
- Consumes: `eval::{eval, type_check, Env, NameKind, Value, length_value}`, `generics::subst_names`.
- Produces on `Scope`: `pub consts: BTreeMap<String, Value>`, `pub binders: BTreeMap<String, i64>`, `pub frame: Vec<(String /*label*/, i64 /*value*/, String /*binder*/)>`; methods `Scope::names(&self) -> BTreeMap<String, NameKind>` (consts + binders + Int/Length generics from `subst`) and `Scope::array_lens(&self) -> BTreeMap<String, i64>`; on `Expander`: `fn eval_int(&mut self, e: &Expr, scope: &Scope, what: &str) -> Option<i64>`, `fn eval_length(&mut self, e: &Expr, scope: &Scope, what: &str) -> Option<UnitValue>` (literal keeps its text; computed gets canonical text), `fn frame_suffix(&self, scope: &Scope) -> String` (`" — in Board::__for_links_9, n = 9"`, empty at top level; appended to every E202/E1402/E1403/E1404 message produced through these helpers).

Pass ordering inside `walk_body` becomes: **pass 0** — collect this body's `const` statements and array declarations into a dependency evaluator: build `pending: BTreeMap<String, ConstOrLen>` (const → its `Expr`; array `NAME` → its `array_len` `Expr`), evaluate each on demand with a recursion stack; a name on the stack again is **E1407** naming the full cycle (`const N: Int = leds.len` + `inst leds: [T; N]` → "`N` → `leds.len` → `N`"); results land in `scope.consts` and a local `lens: BTreeMap<String, i64>` consumed by pass 1. A zero/negative evaluated array length is **E211** at the length expression span ("array length `0` must be 1 or more (computed from `<expr>`)"). Duplicate const names, or a const colliding with an instance/array/binding/label, are **E201**.

- [ ] **Step 1: Write failing tests** (`tests/rfc033_arrays.rs`)

```rust
#[test]
fn computed_length_index_and_len() {
    let src = format!("{LIB}
design Board {{
    const N: Int = 2 * 2
    inst c: [C100N; N]
    inst h: HOST
    net A: h.P, c[0..=(c.len - 1)].A
    net B: h.Q, c[N - 4].B, c[1, 2, 3].B
}}");
    let (chk, r) = check(&src); assert!(!chk.diags.has_errors(), "{r}");
    assert_eq!(netlist(&src).matches("(ref C").count(), 4);
}

#[test]
fn out_of_bounds_computed_index_names_value() {
    let src = format!("{LIB}
design Board {{ const N: Int = 3  inst c: [C100N; N]  inst h: HOST  net A: h.P, c[N].A  nc: h.Q, c[0..=2].B, c[0..=1].A }}");
    let r = check(&src).1;
    assert!(r.contains("E202") && r.contains("index 3 is out of bounds for `c`"), "{r}");
}

#[test]
fn const_cycle_and_bad_length() {
    let src = format!("{LIB}
design Board {{ const N: Int = c.len  inst c: [C100N; N]  inst h: HOST  net _: h.P, h.Q }}");
    let r = check(&src).1; assert!(r.contains("E1407") && r.contains("`N`") && r.contains("`c`"), "{r}");
    let src = format!("{LIB}
design Board {{ const N: Int = 1 - 1  inst c: [C100N; N]  inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E211"));
}

#[test]
fn computed_placement_and_rotate() {
    let src = format!("{LIB}
design Board {{
    const P: Length = 4mm
    inst c: [C100N; 3]
    inst h: HOST
    net A: h.P, c[0..=2].A
    net B: h.Q, c[0..=2].B
    layout {{
        place c[1] at (10mm + 1 * P, 2mm / 2) rotate 45 * 2
        place c[0] at (1.00mm + 0mm, -1.5mm)
    }}
}}");
    let (mut chk, r) = check(&src); assert!(!chk.diags.has_errors(), "{r}");
    let art = cohdl::pipeline::build_artifacts(&mut chk, &cohdl::lock::LockState::default()).unwrap();
    let layout = art.layout.unwrap();
    assert!(layout.contains("\"at\": [14, 1]") || layout.contains("[14, 1]"), "{layout}");
    assert!(layout.contains("\"rotate\": 90"), "{layout}");
    // canonical text for a computed value, literal text preserved for a literal
    assert!(layout.contains("1mm") || layout.contains("[1, -1.5]"), "{layout}");
}

#[test]
fn placement_rotate_out_of_range_and_wrong_kind() {
    let src = format!("{LIB}
design Board {{ inst c: C100N  inst h: HOST  net A: h.P, c.A  net B: h.Q, c.B  layout {{ place c at (0mm, 0mm) rotate 360 }} }}");
    assert!(check(&src).1.contains("E1007"));
    let src = format!("{LIB}
design Board {{ inst c: C100N  inst h: HOST  net A: h.P, c.A  net B: h.Q, c.B  layout {{ place c at (0mm, 2) }} }}");
    assert!(check(&src).1.contains("E1401"), "Int where Length expected");
}
```

(Reuse the `LIB`, `check`, `netlist` helpers from Task 6's file — copy them; each test file is self-contained.)

- [ ] **Step 2: Run** `cargo test --test rfc033_arrays` — fails (temporary E1401 from Task 2).

- [ ] **Step 3: Implement.** Add to `Scope`: `consts`, `binders`, `frame` (all `Default`-able; initialize in the three existing `Scope { … }` literals at `expand_design`, `handle_call:1866`, `handle_subdesign_use:2020`). Add:

```rust
impl Scope {
    fn names(&self) -> BTreeMap<String, NameKind> {
        let mut m = crate::check::generics::subst_names(&self.subst);
        for (k, v) in &self.consts { m.insert(k.clone(), NameKind::Const(v.clone())); }
        for (k, v) in &self.binders { m.insert(k.clone(), NameKind::Binder(*v)); }
        m
    }
    fn array_lens(&self) -> BTreeMap<String, i64> {
        self.arrays.iter().map(|(k, (n, _))| (k.clone(), *n)).collect()
    }
}
impl Expander {
    fn frame_suffix(&self, scope: &Scope) -> String {
        match scope.frame.last() { None => String::new(), Some((_, v, binder)) => format!(" — in {}, {} = {}", crate::resolve::short(&scope.path), binder, v) }
    }
    fn eval_int(&mut self, e: &Expr, scope: &Scope, what: &str) -> Option<i64> {
        let names = scope.names(); let lens = scope.array_lens(); let none = BTreeSet::new();
        let mut local = Diagnostics::new();
        let v = eval(e, &Env { names: &names, array_lens: &lens, unknown_arrays: &none }, &mut local);
        self.push_with_suffix(local, scope);
        match v { Some(Value::Int(i)) => Some(i), Some(Value::Length(_)) => { self.diags.push(Diagnostic::error("E1401", e.span(), format!("{} must be an Int, but `{}` is a Length{}", what, expr_text(e), self.frame_suffix(scope)))); None } None => None }
    }
    fn eval_length(&mut self, e: &Expr, scope: &Scope, what: &str) -> Option<UnitValue> {
        if let Some(v) = e.as_length_literal() { return Some(v.clone()); }   // literal spelling preserved
        … same as eval_int but expecting Length → Some(length_value(f)); Int → E1401 "{what} is a `Length` (`mm`) value — `{expr}` is an Int"
    }
    /// Re-push diagnostics from a local batch with the frame suffix appended to the main message.
    fn push_with_suffix(&mut self, local: Diagnostics, scope: &Scope) { for mut d in local.into_iter() { d.message.push_str(&self.frame_suffix(scope)); self.diags.push(d); } }
}
```

(`Diagnostics::into_iter` and a public `message` field: add `pub fn into_iter(self) -> impl Iterator<Item = Diagnostic>` to `src/diag.rs` and make `Diagnostic.message` `pub` if it is not.)

Pass 0 in `walk_body`:

```rust
        // Pass 0 (RFC-033 §3): constants and array lengths, dependency-ordered.
        let mut pending: BTreeMap<String, (Expr, Option<ConstTy>, Span)> = BTreeMap::new();
        for stmt in body {
            match stmt {
                Stmt::Const(c) => { /* E201 if name already in scope/pending; else pending.insert(name, (value, Some(ty), span)) */ }
                Stmt::Inst(i) => if let Some((e, sp)) = &i.array_len { pending.insert(i.name.name.clone(), (e.clone(), None, *sp)); },
                Stmt::SubdesignUse(u) => if let Some((e, sp)) = &u.array_len { pending.insert(u.name.name.clone(), (e.clone(), None, *sp)); },
                _ => {}
            }
        }
        let mut lens: BTreeMap<String, i64> = BTreeMap::new();
        let mut stack: Vec<String> = Vec::new();
        let keys: Vec<String> = pending.keys().cloned().collect();
        for k in keys { self.eval_pending(&k, &pending, scope, &mut lens, &mut stack); }
```

`eval_pending` (memoized): if `scope.consts` or `lens` has `k` → return; if `stack` contains `k` → E1407 "cyclic dependency: `a` → `b` → … → `a`" (push once, at the first participant, mark all as poisoned); push `k`; build an `Env` whose `names` = `scope.names()` plus `NameKind::Unknown` for pending consts not yet evaluated (so a forward reference triggers a recursive `eval_pending` of that name first: pre-scan the `Expr` for `Name`/`Len` identifiers that are in `pending` and evaluate them before evaluating `k`); evaluate; store `Value` in `scope.consts` (type-checked against declared `ConstTy`, mismatch → E1401 "const `X: Int` cannot be assigned the Length `<expr>`") or `lens` (must be `Value::Int` ≥ 1 else E211/E1401); pop. Pass 1 then reads `lens[&inst.name.name]` instead of the literal and inserts `(n, span)` into `scope.arrays` exactly as today.

Selectors: `indexed_local`, `array_bounds`, `expand_member`, `resolve_array_ref` and the placement-segment walk all call `self.eval_int(e, scope, "an index")` for `Single`/`Range` bounds/`step`/`List` entries, then apply the unchanged bounds logic (E202 message unchanged apart from the suffix). `IndexSel::literal_indices()` is no longer used by the expander. Placement: `at.0/at.1` via `eval_length` (the existing unit/geom-range checks then run on the resulting `UnitValue`); `rotate` via `eval_int` then the existing `> 359` check plus `< 0` → same E1007 message; the `PlaceData.rotate: u16` cast happens after the check.

- [ ] **Step 4: Run** `cargo test --test rfc033_arrays && cargo test` — green, existing bytes unchanged (`cargo run -- build examples/rpi-pico2 --emit kicad_pcb` diff-free against a pre-branch build kept in `/tmp`).
- [ ] **Step 5: Commit** `git add src/check/expand.rs src/diag.rs tests/rfc033_arrays.rs && git commit -m "expand: consts, computed array lengths, .len, expression selectors and placements (RFC-033 §3-§4, §6)"`

---

### Task 8: Expander — labelled `for` frames, hygiene, identity, provenance

**Files:**
- Modify: `src/check/expand.rs` `walk_body` pass 2 (`Stmt::For`), `handle_layout` (layout consts/loops), `handle_net` (anonymous-net counter per frame), `handle_call` (call counter per frame), `Scope`
- Test: `tests/rfc033_loops.rs`, `tests/rfc033_identity.rs` (new)

**Interfaces:**
- Produces: `fn handle_for(&mut self, f: &ForStmt, scope: &mut Scope)`, `fn handle_layout_for(&mut self, f: &LayoutFor, scope: &mut Scope)`, `fn enter_frame(&mut self, label: &Ident, binder: &Ident, value: i64, scope: &Scope) -> Scope` returning the child scope with `path = format!("{}::__for_{}_{}", scope.path, label, frame_value_text(value))`, `frame` extended, `binders[binder] = value`, `is_design_body = false`, and **shared views** of the parent's `local_insts/local_subs/arrays/consts/bindings` (clone the maps — they are read-only inside a loop body since declarations are E1406). Per-frame counters: save `self.anon_net_counter` and `self.call_counter`, set both to 0 for the iteration, restore after.
- `fn frame_value_text(v: i64) -> String` = `v.to_string()` for `v >= 0`, `format!("neg{}", v.unsigned_abs())` otherwise.

- [ ] **Step 1: Write failing tests** (`tests/rfc033_loops.rs`)

```rust
const LIB: &str = r#"
pub trait Led { designator_prefix: "D" }
pub device LedDev { pins { VDD: 1 [power_in], GND: 2 [power_in], DIN: 3 [input], DOUT: 4 [output] } }
impl Led for LedDev {}
pub device HostDev { pins { V5: 1 [power_out], GND: 2 [power_in], DATA: 3 [output] } }
pub footprint FP {}
pub part LED: LedDev { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOST: HostDev { primary { mfr: "m", mpn: "host", footprint: FP } }
"#;

fn chain(n: i64) -> String { format!("{LIB}
design Chain {{
    const N: Int = {n}
    inst host: HOST
    inst leds: [LED; N]
    net VCC [5V]: host.V5, leds[0..=(leds.len - 1)].VDD
    net GND [gnd]: host.GND, leds[0..=(leds.len - 1)].GND
    net DATA: host.DATA, leds[0].DIN
    for links: n in 0..(leds.len - 1) {{
        net _: leds[n].DOUT, leds[n + 1].DIN
    }}
    nc: leds[leds.len - 1].DOUT
}}") }

#[test]
fn led_chain_n_1_2_10() {
    for (n, links) in [(1, 0), (2, 1), (10, 9)] {
        let (chk, r) = check(&chain(n)); assert!(!chk.diags.has_errors(), "N={n}:\n{r}");
        let net = netlist(&chain(n));
        // every DOUT→DIN link is its own net: count nets with both a DOUT and a DIN endpoint
        assert_eq!(net.matches("__for_links_").count(), links, "N={n}\n{net}");
    }
}

#[test]
fn off_by_one_names_the_iteration() {
    let src = chain(10).replace("0..(leds.len - 1)", "0..leds.len");
    let r = check(&src).1;
    assert!(r.contains("E202") && r.contains("index 10 is out of bounds") && r.contains("__for_links_9") && r.contains("n = 9"), "{r}");
    assert_eq!(r.matches("error[E202]").count(), 1, "one diagnostic for the one failing iteration:\n{r}");
}

#[test]
fn reversed_and_empty_ranges() {
    let src = chain(3).replace("0..(leds.len - 1)", "2..1");
    assert!(check(&src).1.contains("E1404"));
    let src = chain(1); // 0..0 — empty, still valid
    assert!(!check(&src).0.diags.has_errors());
}

#[test]
fn direct_inst_in_loop_is_e1406_even_when_empty() {
    let src = format!("{LIB}
design B {{ inst host: HOST  for x: i in 0..0 {{ inst extra: LED }}  net _: host.V5, host.GND, host.DATA }}");
    assert!(check(&src).1.contains("E1406"));
}

#[test]
fn sibling_frames_do_not_merge_same_named_nets() {
    let src = format!("{LIB}
design B {{
    inst host: HOST
    inst leds: [LED; 2]
    net P: host.V5, leds[0..=1].VDD
    net G [gnd]: host.GND, leds[0..=1].GND
    net D: host.DATA, leds[0].DIN
    for a: i in 0..1 {{ net LINK: leds[0].DOUT, leds[1].DIN }}
    for b: i in 0..1 {{ net LINK: leds[1].DOUT }}
}}");
    let (chk, r) = check(&src); assert!(!chk.diags.has_errors(), "{r}");
    let net = netlist(&src);
    assert!(net.contains("__for_a_0::LINK") && net.contains("__for_b_0::LINK"), "{net}");
}

#[test]
fn layout_loop_places_each_element() {
    let src = format!("{LIB}
design B {{
    inst host: HOST
    inst leds: [LED; 6]
    net P: host.V5, leds[0..=5].VDD
    net G [gnd]: host.GND, leds[0..=5].GND
    net D: host.DATA, leds[0..=5].DIN
    nc: leds[0..=5].DOUT
    layout {{
        for grid: n in 0..leds.len {{
            place leds[n] at (10mm + (n % 5) * 4mm, 10mm + (n / 5) * 4mm)
        }}
        place leds[2] at (0mm, 0mm)
    }}
}}");
    let r = check(&src).1;
    assert!(r.contains("E1007") && r.contains("placed more than once"), "loop + explicit duplicate:\n{r}");
    let src2 = src.replace("        place leds[2] at (0mm, 0mm)\n", "");
    let (mut chk, r) = check(&src2); assert!(!chk.diags.has_errors(), "{r}");
    let art = cohdl::pipeline::build_artifacts(&mut chk, &cohdl::lock::LockState::default()).unwrap();
    let layout = art.layout.unwrap();
    assert!(layout.contains("[26, 10]") && layout.contains("[10, 14]"), "{layout}");
}
```

`tests/rfc033_identity.rs`: build the 10-LED chain, keep the returned `LockState`, then (a) insert a second labelled loop above `links` and rebuild with the prior lock → every `__for_links_*` net and every LED designator unchanged; (b) grow `N` to 12 → `__for_links_0..8` survive, `__for_links_9`,`_10` are new; (c) rename the label `links`→`chain` → all `__for_links_*` paths are gone and `__for_chain_*` appear (tombstones for fn-locals are not involved here, so assert on netlist net names only).

- [ ] **Step 2: Run** `cargo test --test rfc033_loops --test rfc033_identity` — fails.

- [ ] **Step 3: Implement.**

```rust
    fn handle_for(&mut self, f: &ForStmt, scope: &mut Scope) {
        // Static admission first: a loop body may not declare.
        for s in &f.body {
            match s {
                Stmt::Inst(i) => self.diags.push(Diagnostic::error("E1406", i.span, "`inst` is not admitted inside a `for` body — declare the array outside the loop and repeat only connections, calls and placements here (RFC-033 Candidate A)")),
                Stmt::SubdesignUse(u) => if scope.place_ctx == PlaceCtx::Fn { /* E1307 wins */ self.diags.push(Diagnostic::error("E1307", u.span, "…existing message…")) } else { self.diags.push(Diagnostic::error("E1406", u.span, "a `subdesign` use site is not admitted inside a `for` body — declare it outside the loop")) },
                _ => {}
            }
        }
        if !self.check_not_reserved(&f.label, "loop label") || !self.check_not_reserved(&f.binder, "loop variable") { return; }
        // Label/binder collisions with anything visible.
        for id in [&f.label, &f.binder] {
            if scope.local_insts.contains_key(&id.name) || scope.local_subs.contains_key(&id.name) || scope.arrays.contains_key(&id.name)
                || scope.bindings.contains_key(&id.name) || scope.consts.contains_key(&id.name) || scope.binders.contains_key(&id.name)
                || scope.frame.iter().any(|(l, _, _)| l == &id.name) {
                self.diags.push(Diagnostic::error("E201", id.span, format!("`{}` is already defined in this scope", id.name))); return;
            }
        }
        let Some(lo) = self.eval_int(&f.start, scope, "a loop bound") else { return };
        let Some(hi) = self.eval_int(&f.end, scope, "a loop bound") else { return };
        if lo > hi {
            self.diags.push(Diagnostic::error("E1404", f.start.span().to(f.end.span()), format!("loop `{}` has a reversed range {}..{} — the end must not be below the start (a half-open range with equal bounds is empty){}", f.label.name, lo, hi, self.frame_suffix(scope))));
            return;
        }
        // Frame depth is part of the budget (Task 10); iterations charged there too.
        for v in lo..hi {
            let mut inner = self.enter_frame(&f.label, &f.binder, v, scope);
            let saved = (self.anon_net_counter, self.call_counter);
            self.anon_net_counter = 0; self.call_counter = 0;
            self.walk_body(&f.body, &mut inner);
            self.anon_net_counter = saved.0; self.call_counter = saved.1;
        }
    }

    fn enter_frame(&mut self, label: &Ident, binder: &Ident, value: i64, scope: &Scope) -> Scope {
        let mut inner = Scope {
            design_name: scope.design_name.clone(),
            path: format!("{}::__for_{}_{}", scope.path, label.name, frame_value_text(value)),
            is_design_body: false,
            place_ctx: scope.place_ctx,
            subst: scope.subst.clone(),
            bindings: scope.bindings.clone(),
            local_insts: scope.local_insts.clone(),
            local_subs: scope.local_subs.clone(),
            arrays: scope.arrays.clone(),
            consts: scope.consts.clone(),
            binders: scope.binders.clone(),
            frame: scope.frame.clone(),
        };
        inner.binders.insert(binder.name.clone(), value);
        inner.frame.push((label.name.clone(), value, binder.name.clone()));
        inner
    }
```

Nets declared inside a frame get `scoped:` keys through the existing `(Some(name), false)` arm because `is_design_body` is false; their display names become `__for_links_3::LINK` after the design-name prefix strip — exactly RFC §7. A frame body's `layout {}` (allowed subset) goes through `handle_layout` with `scope.place_ctx` unchanged, so design-level placements from loops are absolute and subdesign-level ones are relative, and duplicates are caught by the existing `placements.iter().any(|p| p.path == path)` check (E1007). `Placement` targets resolve through `scope.local_insts` (cloned view), so `place leds[n]` finds the parent's array. `handle_layout` gains: evaluate `block.consts` into a local scope clone (layout consts are visible only in that layout and its loops), then `for lf in &block.loops { self.handle_layout_for(lf, &mut layout_scope) }` where `handle_layout_for` mirrors `handle_for` (E1404, frames) but iterates `consts → placements → nested loops`. `walk_body` pass 2 adds `Stmt::For(f) => self.handle_for(f, scope)` and pass 0 ignores `Stmt::Const` inside loops (they are collected when the loop frame's own `walk_body` runs — consts declared in a loop body are frame-local automatically).

Nested loops: `handle_for` is re-entrant through `walk_body`; the frame vector grows; the 64-frame limit is enforced in Task 10.

- [ ] **Step 4: Run** `cargo test --test rfc033_loops --test rfc033_identity && cargo test` — green.
- [ ] **Step 5: Commit** `git add src/check/expand.rs tests/rfc033_loops.rs tests/rfc033_identity.rs && git commit -m "expand: labelled for frames with hygienic identity and iteration provenance (RFC-033 §5-§7)"`

---

### Task 9: Uniform static declaration validation (`bodies.rs`)

**Files:**
- Modify: `src/check/bodies.rs:68-165` (check_one), new helpers; `src/check/mod.rs:25-30` (call `bodies::check_design_bodies`)
- Test: `tests/rfc033_static.rs` (new)

**Interfaces:**
- Consumes: `eval::{type_check, Env, NameKind, Ty}`.
- Produces: `pub fn check_design_bodies(world: &World, diags: &mut Diagnostics)` (runs `check_one` over every design body with `allow_sub_use = true`); inside `check_one`, a recursive `fn check_stmts(ctx: &mut StaticCtx, stmts: &[Stmt], in_loop: bool, diags)` where

```rust
struct StaticCtx<'a> {
    world: &'a World,
    f: &'a FnDef,
    bases: BTreeMap<&'a str, Base<'a>>,          // existing
    names: BTreeMap<String, NameKind>,           // consts (Unknown(ty) unless literal), binders (Unknown(Int)), Int/Length generics (Unknown)
    unknown_arrays: BTreeSet<String>,            // every array declared in this body (length not evaluated statically)
    labels: BTreeSet<String>,
    seen_locals: BTreeSet<String>,               // duplicate-local detection (the §8 correction)
}
```

Checks added (all decidable without values): duplicate local instance/array/subdesign/const/label/binder names → **E201** (this is the enumerated compatibility correction — it now fires for uncalled fns too); `const` value `type_check` against declared `ConstTy` → E1401/E1402/E1403 (a literal `1 / 0` in an empty loop fails here); `for` bounds `type_check` must be `Ty::Int` → E1401; `inst`/`subdesign` inside a `for` → **E1406** (E1307 first in fn bodies); layout loops containing anything but const/place/for → E1406 (parser already rejects; keep the check for AST built by other producers); `.len` on a non-array name → E202; a `Length`-typed const used as an index → E1401 (via `type_check` on every selector expression: `IndexSel` parts, `array_len`, placement segment indexes must type-check to `Int`; `Placement.at` to `Length`, `rotate` to `Int`). Known generic mismatches at definition (`inst r: SeriesR<C>` where `C: Capacitance` bound but `SeriesR` wants `Resistance`) already produce E112 via `check_named_generic_args`; extend it so an Int-generic name given to a unit parameter is E112 and a unit generic given to `const N: Int` is E1401.

- [ ] **Step 1: Write failing tests** (`tests/rfc033_static.rs`)

```rust
#[test]
fn uncalled_fn_duplicate_local_now_fails() {
    let src = format!("{LIB}
pub fn unused(p: Pin) {{ inst c: C100N  inst c: C100N  net _: p, c.A  nc: c.B }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E201"));
}

#[test]
fn empty_loop_hides_nothing_decidable() {
    let src = format!("{LIB}
pub fn f<const N: Int>(p: Pin) {{ for empty: i in 0..0 {{ const BAD: Int = i / 0  net _: p }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E1403"), "known zero divisor in an uncalled fn's empty loop");
    let src = format!("{LIB}
pub fn g(p: Pin) {{ for empty: i in 0..0 {{ net _: missing.P }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E202"));
    let src = format!("{LIB}
pub fn h(p: Pin) {{ for empty: i in 0..0 {{ const X: Int = 1 / i  net _: p }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}");
    assert!(!check(&src).1.contains("E1403"), "1 / i waits for a real i");
}

#[test]
fn const_type_and_bounds_kinds() {
    let src = format!("{LIB}
design Board {{ const A: Int = 2mm  inst h: HOST  net _: h.P, h.Q }}");
    assert!(check(&src).1.contains("E1401"));
    let src = format!("{LIB}
design Board {{ const L: Length = 2mm  inst h: HOST  for x: i in 0..L {{ net _: h.P }}  net _: h.Q }}");
    assert!(check(&src).1.contains("E1401"));
}

#[test]
fn skipped_activation_is_not_value_specialized() {
    let src = format!("{LIB}
pub fn f<const N: Int>(p: Pin) {{ for e: i in 0..0 {{ const BAD: Int = 1 / N  net _: p }} }}
design Board {{ inst h: HOST  for none: i in 0..0 {{ f::<0>(h.P) }}  net _: h.P, h.Q }}");
    let r = check(&src).1;
    assert!(!r.contains("E1403"), "no callee specialization inside a skipped loop:\n{r}");
    let src2 = src.replace("f::<0>(h.P)", "f::<1 / 0>(h.P)");
    assert!(check(&src2).1.contains("E1403"), "the argument expression itself is checked");
    let src3 = format!("{LIB}
pub fn f<const N: Int>(p: Pin) {{ for e: i in 0..0 {{ const BAD: Int = 1 / N  net _: p }} }}
design Board {{ inst h: HOST  f::<0>(h.P)  net _: h.Q }}");
    assert!(check(&src3).1.contains("E1403"), "actual activation binds N=0 and fails");
}
```

- [ ] **Step 2: Run** `cargo test --test rfc033_static` — fails.
- [ ] **Step 3: Implement** `check_stmts` recursion (call it from `check_one` in place of the flat loop; `Stmt::For` → validate label/binder uniqueness, `type_check` bounds, recurse with `in_loop = true` and the binder added as `NameKind::Unknown(Ty::Int)`; `Stmt::Const` → record, `type_check`; `Stmt::Layout` → type-check placement expressions and recurse into `loops`). For the activation case (`src3`), the expander's `handle_call` already binds `N = 0` and walks the body: `handle_for` on `0..0` must still run pass 0 of the loop body statically under the bound substitution — implement by having `handle_for` call `bodies::check_loop_body_bound(world, &f.body, &scope.names(), diags)` (type_check + concrete-subexpression eval with the frame's known names) **before** the range decision, once per loop entry, not per iteration. Register `check_design_bodies` in `check_declarations` after `check_fn_bodies`.
- [ ] **Step 4: Run** `cargo test` — green. Note any pre-existing test that relied on an uncalled duplicate local passing; there should be none (grep `inst c: .*\n.*inst c:` in tests).
- [ ] **Step 5: Commit** `git add src/check tests/rfc033_static.rs && git commit -m "check: uniform static declaration validation for consts, loops and duplicates (RFC-033 §8)"`

---

### Task 10: Metering — activation walk and deterministic budgets

**Files:**
- Create: `src/check/meter.rs`; register in `src/check/mod.rs`
- Modify: `src/check/expand.rs` (charge points; `expand_design` computes activation; frame depth)
- Test: `tests/rfc033_budget.rs` (new)

**Interfaces:**
- Produces:

```rust
pub const MAX_ITERATIONS: u64 = 100_000;
pub const MAX_WORK_ITEMS: u64 = 1_000_000;
pub const MAX_FRAMES: usize = 64;
/// True when the syntactic expansion graph rooted at `design` contains any M2 construct.
pub fn metering_needed(world: &World, design: &DesignDef) -> bool;
pub struct Meter { pub active: bool, pub iterations: u64, pub work: u64, pub frames: usize, tripped: bool }
impl Meter {
    pub fn new(active: bool) -> Meter;
    /// Charge `n` work items; on the first overflow push E1405 (once) and return false.
    pub fn charge(&mut self, n: u64, what: &str, span: Span, diags: &mut Diagnostics) -> bool;
    pub fn enter_iteration(&mut self, span: Span, diags: &mut Diagnostics) -> bool;
    pub fn enter_frame(&mut self, span: Span, diags: &mut Diagnostics) -> bool;  // depth check
    pub fn leave_frame(&mut self);
}
```

`metering_needed` walks: the design body; for each `Stmt::Call` the callee fn body (visited set on fq name); for each `Stmt::SubdesignUse` the subdesign body (visited set), including their signatures (`GenericBound::Int` anywhere, `GenericDefault::Int`); returns true on the first `Stmt::Const`, `Stmt::For`, `LayoutBlock.consts/loops`, any `Expr` that is not a bare literal (array lengths, selectors, placements, generic args `GenericArg::Expr`), or `Expr::Len`. Device/part references contribute nothing.

Charge points in the expander, each `if !self.meter.charge(..) { return; }` **before** the corresponding push/insert: `handle_inst` (1 per real instance, after the duplicate check), `handle_call` (1 per entered call, before `walk_body`), `handle_subdesign_use` (1 per node + 1 per declared port, before the node stub insert), `handle_subdesign_conns` (1 per entry + 1 per scalar target), `handle_net`/`handle_nc` (1 per statement + 1 per expanded member before dedup), `handle_placement` (1), `handle_layout` constraints (1 per declaration + 1 per net reference), physics records in `handle_inst_phys`/`handle_net` (1 per record + 1 per target). `handle_for`/`handle_layout_for`: `enter_frame` before the iteration loop (E1405 "loop nesting exceeds 64 active frames"), `enter_iteration` per iteration, `leave_frame` after. When the meter is inactive every method returns true without counting.

- [ ] **Step 1: Write failing tests** (`tests/rfc033_budget.rs`) — use nested loops with a single-member `nc` so counts are cheap: 2 iterations × 8 work items example from the RFC, then exact boundaries. `for a: i in 0..1000 { for b: j in 0..100 { … } }` = 100,000 iterations (passes); `0..1001` × `0..100` = 100,100 (E1405 naming `100,001` and the limit). Work items: a loop of 500,000 iterations each charging one `nc` statement + one member = 1,000,000 (passes at exactly the limit); one more iteration fails. Depth: 64 nested empty-range loops pass; 65 fail. Also `legacy_graph_not_metered`: a design with no M2 syntax but 1,200,000 literal-array elements is too slow to build — instead assert `metering_needed` is false via a design with 10 literal instances and check that a loop-free program never produces E1405 even when a helper is `pub fn` with M2 syntax but is **not** referenced (`unrelated uncalled M2 definition`), while referencing it from an empty loop **does** activate (assert through a work-item overflow made cheap: reference the helper and also declare a `[T; 1000001]` literal array → E1405 only in the activated case).

- [ ] **Step 2: Run** `cargo test --test rfc033_budget` — fails.
- [ ] **Step 3: Implement** `meter.rs`; add `meter: Meter` to `Expander`, initialized in `expand_design` with `Meter::new(metering_needed(world, design))`; wire the charge points; E1405 message: `format!("{} would exceed the deterministic expansion budget: {} {} (limit {}){}", what, count, unit, limit, frame_suffix)` with unit `iterations`/`work items`/`active frames`; push once (`tripped`), then every subsequent `charge` returns false so no partial materialization continues. `expand_design` returns the IR only when `!meter.tripped` (assembly is skipped after a trip; the diag already carries the site).
- [ ] **Step 4: Run** `cargo test --test rfc033_budget && cargo test` — green; time the budget tests (`--nocapture` shows < 5 s total; if the 1,000,000-item case is slow, charge before allocation as specified so no instances are created).
- [ ] **Step 5: Commit** `git add src/check/meter.rs src/check/mod.rs src/check/expand.rs tests/rfc033_budget.rs && git commit -m "check: expansion metering with activation walk and E1405 budgets (RFC-033 §9)"`

---

### Task 11: Error registry, ledger, specification snapshot

**Files:**
- Modify: `docs/error-codes.md` (new `## E14xx` block after E13xx at `:242-258`; widen E201 row at the E2xx block), `docs/compliance-report.md` (append an RFC-033 ledger entry), `docs/design/10-language-specification.md` (new section "Parameterized circuit construction (RFC-033)" before `# Not yet specified` at `:1108`), `AGENTS.md` (one paragraph in the "Originally-cut items … ARE implemented" list, RFC-033 summary)
- Test: `tests/error_registry.rs` (existing; both directions)

- [ ] **Step 1: Run** `cargo test --test error_registry` — after Tasks 3–10 it FAILS listing E1401–E1407 as emitted-but-unregistered.
- [ ] **Step 2: Add the registry block:**

```markdown
## E14xx — parameterized circuit construction (RFC-033)

| Code | Meaning |
| --- | --- |
| E1401 | expected a compile-time `Int` or `Length` (or a supported operand pairing) — a unit literal where a count is required, an Int where a `Length` coordinate is required, `Int + Length`, `Length * Length`, a non-`Int`/`Length` const type, or an expression given to a non-Int/Length generic parameter |
| E1402 | Int or Length overflow, `MIN / -1`, `MIN % -1`, an out-of-range integer literal, or a `Length / Int` that is not exactly representable (Length arithmetic never rounds) |
| E1403 | division or remainder by zero, including a known zero divisor inside a loop body that never runs |
| E1404 | reversed `for` range — the end is below the start (equal bounds are an empty loop, not an error) |
| E1405 | deterministic expansion budget exceeded: 100,000 entered iterations, 1,000,000 work items, or 64 active loop frames; reported before the excess object is materialized, never a partial build |
| E1406 | a declaration or operation not admitted in this context: `inst`/`subdesign` inside a `for` body (including an empty one), or anything but `const`/`place`/`for` inside a layout loop |
| E1407 | cyclic constant / array-length dependency — the complete cycle is named (`N` → `leds.len` → `N`) |
```

E201 row: replace "duplicate top-level declaration" with "duplicate declaration — a top-level name declared twice at one module path, or a local name (instance, array, subdesign use site, const, loop label or loop variable) that collides with anything visible in its scope".

- [ ] **Step 3: Ledger entry** in `docs/compliance-report.md` (same shape as the RFC-032 entry): implemented scope (Candidate A), the one compatibility correction (uniform duplicate-local/generic-mismatch validation, with the corpus audit numbers from Task 14), the deviations: `Placement.rotate` accepts any `0..=359` (existing deviation, now expression-valued), `fmt` prints loops in source order, `__for_` frames reset the anonymous-net and fn-call counters per iteration (paths stay injective through the frame segment), docs schema v2 rule.
- [ ] **Step 4: Note 10 section** — copy RFC-033 §1–§10 Rules in the specification's house style (an "Accepted via RFC-033" preamble, code blocks for the LED/RC/nested scenarios, the E14xx table). Update the "Not yet specified" bullet about a general loop construct to say it is now RFC-033, with Candidate B (loop-body declarations) still deferred.
- [ ] **Step 5: Run** `cargo test --test error_registry && cargo test` — green.
- [ ] **Step 6: Commit** `git add docs AGENTS.md && git commit -m "docs: register E1401-E1407, ledger and note 10 section for RFC-033"`

---

### Task 12: LSP — hover and symbols for consts, binders, `.len`

**Files:**
- Modify: `src/lsp.rs:806-900` (hover), `:1044`, `:1954` (statement walks)
- Test: `tests/lsp.rs` (append two tests using the existing harness helpers in that file: grep `fn hover_at` / `fn open_and_hover`)

**Interfaces:**
- Consumes: `Stmt::Const/For`, `LayoutBlock.consts/loops`, `expr_text`.
- Produces: hover on a `const` name → "`const N: Int`" plus "value: 10" only when the const lives in a design body and evaluates with no unknowns (use `eval` with the design's own consts; generic/loop references show no value); hover on a loop binder → "loop variable of `for links` (Int)"; hover on `ARRAY.len` → "length of array `leds`: 10" (design-level literal/const lengths only). Document symbols: loops appear as children (`for links`) with their body statements nested; consts as symbols of kind Constant.

- [ ] **Step 1: Write failing tests** in `tests/lsp.rs`: open a design with `const N: Int = 4`, `inst leds: [LED; N]`, `for links: n in 0..N - 1 { … }`; assert hover text on `N` contains `const N: Int` and `= 4`; hover on `n` inside the loop contains `loop variable`; document symbols contain a `for links` entry.
- [ ] **Step 2: Run** `cargo test --test lsp rfc033` — fails.
- [ ] **Step 3: Implement** by extending the two statement walkers to descend into `Stmt::For(f).body` and `LayoutBlock.loops`, adding `Stmt::Const` symbol emission, and a hover arm that resolves the identifier under the cursor against an enclosing-body const/binder table (build it while walking to the cursor: a `Vec<(String, ConstStmt|binder)>` stack).
- [ ] **Step 4: Run** `cargo test --test lsp` — green.
- [ ] **Step 5: Commit** `git add src/lsp.rs tests/lsp.rs && git commit -m "lsp: hover and symbols for consts, loop binders and .len (RFC-033 tooling)"`

---

### Task 13: Package API docs `schema_version: 2`

**Files:**
- Modify: `src/emit/docsjson.rs:28` (SCHEMA_VERSION → per-document decision), `:155-175` (generics_val), `:186-235` (body_summary), `:960-970` (envelope)
- Modify: `docs/apidocs.md` (v2 contract paragraph), `registry/` envelope validator if it pins `schema_version == 1` (grep `schema_version` under `registry/src`; accept `1 | 2`)
- Test: `tests/rfc033_docs.rs` (new) + existing `tests/apidocs.rs` stays green (v1 documents unchanged byte-for-byte)

**Interfaces:**
- Produces: `fn item_uses_m2(item) -> bool` (any `GenericBound::Int`, `Stmt::Const/For`, layout consts/loops, or non-literal `Expr` in its signature/body); document `schema_version` = 2 iff any emitted local or foreign item uses M2, else 1 (byte-identical to today). In v2: Int generic → `"bound": {"const": "Int"}`, `"default": "10"`; an M2 item carries `"body_source": "<canonical body text>"` and omits `insts`/`calls`/`nets`; generic-argument strings use `expr_text`.
- `body_source` is produced by formatting the whole source file with `crate::fmt::format_source`, re-parsing the formatted text, locating the same item by fq name, and slicing the formatted text from the body's first `{` to its matching `}` (spans are byte offsets into the formatted text, so this is exact and uses the one formatter).

- [ ] **Step 1: Write failing tests** (`tests/rfc033_docs.rs`): use `cohdl::emit::docsjson` the way `tests/apidocs.rs` does (copy its `docs_for(files)` helper); assert a package with one `pub fn bank<const N: Int = 2>(p: Pin) { for x: i in 0..N { net _: p } }` yields `"schema_version": 2`, `"bound": {"const": "Int"}`, `"default": "2"`, a `"body_source"` containing `for x: i in 0..N {`, and no `"nets"` key on that item; assert a package without M2 still yields `"schema_version": 1` with unchanged bytes versus a golden captured before the change (`tests/apidocs.rs` already pins several).
- [ ] **Step 2: Run** — fails.
- [ ] **Step 3: Implement**; update `docs/apidocs.md`; if `registry/` has a TypeScript validator with `schema_version !== 1`, change it to accept 1 or 2 and render `body_source` as escaped preformatted text (do not evaluate anything).
- [ ] **Step 4: Run** `cargo test --test rfc033_docs --test apidocs` — green.
- [ ] **Step 5: Commit** `git add src/emit/docsjson.rs docs/apidocs.md tests/rfc033_docs.rs registry && git commit -m "docs: API docs schema v2 for parameterized items (RFC-033 tooling)"`

---

### Task 14: Scenario fixtures, corpus audit, examples re-check

**Files:**
- Create: `tests/rfc033_scenarios.rs` (the three normative scenarios with synthetic parts: LED chain N=1/2/10; RC channels 4N+1 instances / 2N+1 net classes with the capacitor-seven override, then N=12 with a carried `LockState`; nested `FilterBank<const N: Int, R, C>` with `bank.channels[1].c` override — assert 11 instances / 5 net classes for N=3)
- Create: `docs/proposals/fixtures/m2-programmability/README.md` update marking `*.cohdl.txt` as now-runnable and pointing at the tests
- Modify: `docs/compliance-report.md` (audit numbers)

- [ ] **Step 1: Write the scenario tests** by transcribing RFC-033 §Scenarios verbatim into `.cohdl` source strings over a synthetic `LIB` (devices `AddressableLED`, `Host`, `SeriesR<R: Resistance>`, `ShuntC<C: Capacitance>`, `SignalSource`, `SignalSink`, `Ground`, each with a `pub part` so `build` is part-bound). Assertions: instance and net-class counts (`ir.instances.len()`, distinct net names in the netlist), `layout.json` contains `channels_6::c` at `[62, 17]` and `channels_1::c`/`channels_0::c` at their defaults, lock designators for `channels_0::r`…`channels_9::c` identical between the N=10 and N=12 builds when the N=10 lock is passed as `prior_lock`.
- [ ] **Step 2: Run** `cargo test --test rfc033_scenarios` — green (everything landed in Tasks 7–10; if a count is off, the scenario transcription is wrong, not the RFC).
- [ ] **Step 3: Corpus audit** (the §Compatibility gate): build the pre-branch compiler once (`git worktree add /tmp/cohdl-base origin/main && cargo build --manifest-path /tmp/cohdl-base/Cargo.toml`), then for every package directory under `lib/` (and `lib/@*/**`) and every `examples/*` run both `check` binaries and diff verdict + diagnostics:

```bash
for d in $(find lib examples -name cohdl.toml -exec dirname {} \; | sort); do
  a=$(/tmp/cohdl-base/target/debug/cohdl check "$d" --json 2>/dev/null | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d["verdict"],sorted(x["code"] for x in d["diagnostics"]))' 2>/dev/null)
  b=$(target/debug/cohdl check "$d" --json 2>/dev/null | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d["verdict"],sorted(x["code"] for x in d["diagnostics"]))' 2>/dev/null)
  [ "$a" = "$b" ] || echo "DIFF $d: $a -> $b"
done
```

Record the package count, file count, and every `DIFF` line (expected: only E201 duplicate-local corrections, ideally zero) in the ledger entry from Task 11. Also `cargo run -- build examples/rpi-pico2 --emit kicad_pcb` etc. for all three examples and `cmp` every `out/` artifact against the base build — byte-identical.

- [ ] **Step 4: Run the full gate** `cargo test && cargo run -- fmt lib examples book/examples --check && cargo build -p cohdl-explorer-extractor --manifest-path explorer/extractor/Cargo.toml` — all green.
- [ ] **Step 5: Commit** `git add tests/rfc033_scenarios.rs docs && git commit -m "tests: RFC-033 normative scenarios; corpus audit recorded"`

---

## Self-review

**Spec coverage** (RFC-033 section → task): §2 expression model → Tasks 3, 5; §3 consts + Int generics → Tasks 6, 7, 9; §4 array evaluator/`.len`/selectors → Task 7; §5 labelled half-open `for`, admitted bodies, E1406 → Tasks 3, 8, 9; §6 placement → Tasks 7, 8; §7 identity frames → Task 8 (+ identity tests); §8 static checks / activation / iteration → Tasks 5, 9, 8; §9 metering → Task 10; §10 lexing + diagnostics block + provenance → Tasks 1, 3, 7 (`frame_suffix`), 11; Scenarios → Task 14; Tooling (fmt/LSP/docs v2/Explorer) → Tasks 4, 12, 13, 14 step 4; Compatibility audit → Task 14; Rules 1–14 are each enforced by at least one test above.

**Known gaps to raise at review, not silently decided:** (1) `-40C` as a spec/generic default is handled by `signed_unit_literal`, while `Temperature` never enters `expr()` — confirm no existing source spells a Temperature inside an expression position. (2) The per-iteration reset of the fn-call counter changes nothing for legacy source (no frames) but must be stated in the ledger. (3) Registry TypeScript changes live outside the Rust crate's CI; note in the PR.

**Type consistency check:** `Expr`, `ConstStmt`, `ForStmt`, `LayoutFor`, `GenericDefault`, `GenericArg::Expr`, `GenericValue::Int`, `NameKind`, `Env`, `Value`, `Ty`, `Meter`, `frame_suffix`, `eval_int`, `eval_length`, `enter_frame`, `frame_value_text`, `subst_names`, `check_design_bodies`, `metering_needed` are named identically in every task that uses them.
