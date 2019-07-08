use crate::parser::*;
use nom::branch::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum NetLvalue<'a> {
    Identifier(Box<NetLvalueIdentifier<'a>>),
    Lvalue(Box<Vec<NetLvalue<'a>>>),
    Pattern(Box<NetLvaluePattern<'a>>),
}

#[derive(Debug)]
pub struct NetLvalueIdentifier<'a> {
    pub nodes: (PsOrHierarchicalNetIdentifier<'a>, ConstantSelect<'a>),
}

#[derive(Debug)]
pub struct NetLvaluePattern<'a> {
    pub nodes: (
        Option<AssignmentPatternExpressionType<'a>>,
        AssignmentPatternNetLvalue<'a>,
    ),
}

#[derive(Debug)]
pub enum VariableLvalue<'a> {
    Identifier(Box<VariableLvalueIdentifier<'a>>),
    Lvalue(Box<Vec<VariableLvalue<'a>>>),
    Pattern(Box<VariableLvaluePattern<'a>>),
    Concatenation(Box<StreamingConcatenation<'a>>),
}

#[derive(Debug)]
pub struct VariableLvalueIdentifier<'a> {
    pub nodes: (
        Option<ImplicitClassHandleOrPackageScope<'a>>,
        HierarchicalVariableIdentifier<'a>,
        Select<'a>,
    ),
}

#[derive(Debug)]
pub struct VariableLvaluePattern<'a> {
    pub nodes: (
        Option<AssignmentPatternExpressionType<'a>>,
        AssignmentPatternVariableLvalue<'a>,
    ),
}

#[derive(Debug)]
pub struct NonrangeVariableLvalue<'a> {
    pub nodes: (
        Option<ImplicitClassHandleOrPackageScope<'a>>,
        HierarchicalVariableIdentifier<'a>,
        Select<'a>,
    ),
}

// -----------------------------------------------------------------------------

pub fn net_lvalue(s: &str) -> IResult<&str, NetLvalue> {
    alt((net_lvalue_identifier, net_lvalue_lvalue, net_lvalue_pattern))(s)
}

pub fn net_lvalue_identifier(s: &str) -> IResult<&str, NetLvalue> {
    let (s, x) = ps_or_hierarchical_net_identifier(s)?;
    let (s, y) = constant_select(s)?;
    Ok((
        s,
        NetLvalue::Identifier(Box::new(NetLvalueIdentifier { nodes: (x, y) })),
    ))
}

pub fn net_lvalue_pattern(s: &str) -> IResult<&str, NetLvalue> {
    let (s, x) = opt(assignment_pattern_expression_type)(s)?;
    let (s, y) = assignment_pattern_net_lvalue(s)?;
    Ok((
        s,
        NetLvalue::Pattern(Box::new(NetLvaluePattern { nodes: (x, y) })),
    ))
}

pub fn net_lvalue_lvalue(s: &str) -> IResult<&str, NetLvalue> {
    let (s, _) = symbol("{")(s)?;
    let (s, x) = net_lvalue(s)?;
    let (s, y) = many0(preceded(symbol(","), net_lvalue))(s)?;
    let (s, _) = symbol("}")(s)?;

    let mut ret = Vec::new();
    ret.push(x);
    for y in y {
        ret.push(y);
    }

    Ok((s, NetLvalue::Lvalue(Box::new(ret))))
}

pub fn variable_lvalue(s: &str) -> IResult<&str, VariableLvalue> {
    alt((
        variable_lvalue_identifier,
        variable_lvalue_lvalue,
        variable_lvalue_pattern,
        map(streaming_concatenation, |x| {
            VariableLvalue::Concatenation(Box::new(x))
        }),
    ))(s)
}

pub fn variable_lvalue_identifier(s: &str) -> IResult<&str, VariableLvalue> {
    let (s, x) = opt(implicit_class_handle_or_package_scope)(s)?;
    let (s, y) = hierarchical_variable_identifier(s)?;
    let (s, z) = select(s)?;
    Ok((
        s,
        VariableLvalue::Identifier(Box::new(VariableLvalueIdentifier { nodes: (x, y, z) })),
    ))
}

pub fn variable_lvalue_pattern(s: &str) -> IResult<&str, VariableLvalue> {
    let (s, x) = opt(assignment_pattern_expression_type)(s)?;
    let (s, y) = assignment_pattern_variable_lvalue(s)?;
    Ok((
        s,
        VariableLvalue::Pattern(Box::new(VariableLvaluePattern { nodes: (x, y) })),
    ))
}

pub fn variable_lvalue_lvalue(s: &str) -> IResult<&str, VariableLvalue> {
    let (s, _) = symbol("{")(s)?;
    let (s, x) = variable_lvalue(s)?;
    let (s, y) = many0(preceded(symbol(","), variable_lvalue))(s)?;
    let (s, _) = symbol("}")(s)?;

    let mut ret = Vec::new();
    ret.push(x);
    for y in y {
        ret.push(y);
    }

    Ok((s, VariableLvalue::Lvalue(Box::new(ret))))
}

pub fn nonrange_variable_lvalue(s: &str) -> IResult<&str, NonrangeVariableLvalue> {
    let (s, x) = opt(implicit_class_handle_or_package_scope)(s)?;
    let (s, y) = hierarchical_variable_identifier(s)?;
    let (s, z) = nonrange_select(s)?;
    Ok((s, NonrangeVariableLvalue { nodes: (x, y, z) }))
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            format!("{:?}", all_consuming(net_lvalue)("a")),
            "Ok((\"\", Identifier(NetLvalueIdentifier { identifier: ScopedIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } } }, select: ConstantSelect { member: None, bit_select: [], part_select_range: None } })))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(net_lvalue)("a[1][2]")),
            "Ok((\"\", Identifier(NetLvalueIdentifier { identifier: ScopedIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } } }, select: ConstantSelect { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\"))))), Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"2\")))))], part_select_range: None } })))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(net_lvalue)("a[1][10:5]")),
            "Ok((\"\", Identifier(NetLvalueIdentifier { identifier: ScopedIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } } }, select: ConstantSelect { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\")))))], part_select_range: Some(Range((Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"10\"))))), Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"5\")))))))) } })))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(net_lvalue)("{a, b[1], c}")),
            "Ok((\"\", Lvalue([Identifier(NetLvalueIdentifier { identifier: ScopedIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } } }, select: ConstantSelect { member: None, bit_select: [], part_select_range: None } }), Identifier(NetLvalueIdentifier { identifier: ScopedIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"b\" } } }, select: ConstantSelect { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\")))))], part_select_range: None } }), Identifier(NetLvalueIdentifier { identifier: ScopedIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"c\" } } }, select: ConstantSelect { member: None, bit_select: [], part_select_range: None } })])))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(variable_lvalue)("a")),
            "Ok((\"\", Identifier(VariableLvalueIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } }, select: Select { member: None, bit_select: [], part_select_range: None } })))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(variable_lvalue)("a[1][2]")),
            "Ok((\"\", Identifier(VariableLvalueIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } }, select: Select { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\"))))), Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"2\")))))], part_select_range: None } })))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(variable_lvalue)("a[1][10:5]")),
            "Ok((\"\", Identifier(VariableLvalueIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } }, select: Select { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\")))))], part_select_range: Some(Range((Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"10\"))))), Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"5\")))))))) } })))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(variable_lvalue)("{a, b[1], c}")),
            "Ok((\"\", Lvalue([Identifier(VariableLvalueIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } }, select: Select { member: None, bit_select: [], part_select_range: None } }), Identifier(VariableLvalueIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"b\" } }, select: Select { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\")))))], part_select_range: None } }), Identifier(VariableLvalueIdentifier { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"c\" } }, select: Select { member: None, bit_select: [], part_select_range: None } })])))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(nonrange_variable_lvalue)("a")),
            "Ok((\"\", NonrangeVariableLvalue { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } }, select: Select { member: None, bit_select: [], part_select_range: None } }))"
        );
        assert_eq!(
            format!("{:?}", all_consuming(nonrange_variable_lvalue)("a[1][2]")),
            "Ok((\"\", NonrangeVariableLvalue { scope: None, identifier: HierarchicalIdentifier { hierarchy: [], identifier: Identifier { raw: \"a\" } }, select: Select { member: None, bit_select: [Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"1\"))))), Nullary(PrimaryLiteral(Number(IntegralNumber(UnsignedNumber(\"2\")))))], part_select_range: None } }))"
        );
    }
}