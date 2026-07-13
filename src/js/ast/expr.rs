use super::decl::Decorator;
use super::literal::{BoolLit, Lit, NumLit, StrLit, TemplateLit};
use super::node::AstNode;
use super::pattern::Pat;
use super::stmt::BlockStmt;
use crate::span::Span;

pub type ExprRef = crate::arena::NodeId;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Expr {
    Ident(Ident),
    Lit(Lit),
    This(ThisExpr),
    Super(SuperExpr),
    Array(ArrayExpr),
    Object(ObjectExpr),
    Fn(FnExpr),
    Arrow(ArrowExpr),
    Class(ClassExpr),
    New(NewExpr),
    Call(CallExpr),
    OptionalCall(OptionalCallExpr),
    Member(MemberExpr),
    OptionalMember(OptionalMemberExpr),
    Unary(UnaryExpr),
    UnaryOp(UnaryOpExpr),
    Binary(BinaryExpr),
    Logical(LogicalExpr),
    Conditional(ConditionalExpr),
    Assignment(AssignmentExpr),
    Sequence(SequenceExpr),
    Update(UpdateExpr),
    Await(AwaitExpr),
    Yield(YieldExpr),
    Spread(SpreadExpr),
    Template(TemplateLit),
    TaggedTemplate(TaggedTemplateExpr),
    MetaProperty(MetaPropExpr),
    Import(ImportExpr),
    JSXElement(JSXElement),
    JSXFragment(JSXFragment),
    TSAs(TSAsExpr),
    TSSatisfies(TSSatisfiesExpr),
    TSTypeAssertion(TSTypeAssertionExpr),
    TSNonNull(TSNonNullExpr),
    TSInst(TSInstantiationExpr),
    Parenthesized(ParenthesizedExpr),
    PrivateName(PrivateNameExpr),
    Chain(ChainExpr),
    Invalid(InvalidExpr),
    /// Record literal `#{ a: 1, b }` (immutable object).
    Record(RecordExpr),
    /// Tuple literal `#[1, 2, 3]` (immutable array).
    Tuple(TupleExpr),
    /// Pipeline operator `a |> b` (Hack-style, single-arg).
    Pipeline(PipelineExpr),
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub span: Span,
    pub name: String,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct ThisExpr {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SuperExpr {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ArrayExpr {
    pub span: Span,
    pub elements: Vec<Option<ExprRef>>,
}

#[derive(Debug, Clone)]
pub struct ObjectExpr {
    pub span: Span,
    pub props: Vec<ObjProp>,
}

#[derive(Debug, Clone)]
pub enum ObjProp {
    KeyValue(KeyValueProp),
    Shorthand(Ident),
    Method(MethodProp),
    Spread(SpreadExpr),
    Getter(GetterProp),
    Setter(SetterProp),
}

#[derive(Debug, Clone)]
pub struct KeyValueProp {
    pub span: Span,
    pub key: PropName,
    pub value: ExprRef,
}

#[derive(Debug, Clone)]
pub struct MethodProp {
    pub span: Span,
    pub key: PropName,
    pub function: FnExpr,
}

#[derive(Debug, Clone)]
pub struct GetterProp {
    pub span: Span,
    pub key: PropName,
    pub body: Option<BlockStmt>,
}

#[derive(Debug, Clone)]
pub struct SetterProp {
    pub span: Span,
    pub key: PropName,
    pub param: Pat,
    pub body: Option<BlockStmt>,
}

#[derive(Debug, Clone)]
pub enum PropName {
    Ident(Ident),
    Str(StrLit),
    Num(NumLit),
    Computed(ExprRef),
}

#[derive(Debug, Clone)]
pub struct FnExpr {
    pub span: Span,
    pub id: Option<Ident>,
    pub params: Vec<Pat>,
    pub body: Option<BlockStmt>,
    pub generator: bool,
    pub async_: bool,
}

#[derive(Debug, Clone)]
pub struct ArrowExpr {
    pub span: Span,
    pub params: Vec<Pat>,
    pub body: ArrowBody,
    pub async_: bool,
}

#[derive(Debug, Clone)]
pub enum ArrowBody {
    Expr(ExprRef),
    Block(BlockStmt),
}

#[derive(Debug, Clone)]
pub struct ClassExpr {
    pub span: Span,
    pub id: Option<Ident>,
    pub super_class: Option<ExprRef>,
    pub body: Vec<ClassMember>,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Method(MethodDef),
    Getter(GetterProp),
    Setter(SetterProp),
    Prop(ClassProp),
    Ctor(CtorDef),
    StaticBlock(StaticBlock),
    TSIndex(TsIndexSig),
}

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub span: Span,
    pub key: PropName,
    pub function: FnExpr,
    pub is_static: bool,
    pub kind: MethodKind,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone)]
pub enum MethodKind {
    Method,
    Get,
    Set,
}

#[derive(Debug, Clone)]
pub struct ClassProp {
    pub span: Span,
    pub key: PropName,
    pub value: Option<ExprRef>,
    pub is_static: bool,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone)]
pub struct CtorDef {
    pub span: Span,
    pub params: Vec<Pat>,
    pub body: Option<BlockStmt>,
}

#[derive(Debug, Clone)]
pub struct StaticBlock {
    pub span: Span,
    pub body: BlockStmt,
}

#[derive(Debug, Clone)]
pub struct TsIndexSig {
    pub span: Span,
    pub key: Box<Pat>,
    pub value: Box<TypeAnn>,
}

#[derive(Debug, Clone)]
pub struct NewExpr {
    pub span: Span,
    pub callee: ExprRef,
    pub args: Vec<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub span: Span,
    pub callee: ExprRef,
    pub args: Vec<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct OptionalCallExpr {
    pub span: Span,
    pub callee: ExprRef,
    pub args: Vec<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub span: Span,
    pub object: ExprRef,
    pub property: Box<Expr>,
    pub computed: bool,
}

#[derive(Debug, Clone)]
pub struct OptionalMemberExpr {
    pub span: Span,
    pub object: ExprRef,
    pub property: Box<Expr>,
    pub computed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Plus,
    Not,
    BitNot,
    Typeof,
    Void,
    Delete,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub span: Span,
    pub op: UnaryOp,
    pub arg: ExprRef,
}

#[derive(Debug, Clone)]
pub struct UnaryOpExpr {
    pub span: Span,
    pub op: UnaryOp,
    pub arg: ExprRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    EqEq,
    NotEq,
    EqEqEq,
    NotEqEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    LShift,
    RShift,
    RShift3,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    In,
    Instanceof,
    StarStar,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub span: Span,
    pub op: BinaryOp,
    pub left: ExprRef,
    pub right: ExprRef,
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
    Nullish,
}

#[derive(Debug, Clone)]
pub struct LogicalExpr {
    pub span: Span,
    pub op: LogicalOp,
    pub left: ExprRef,
    pub right: ExprRef,
}

#[derive(Debug, Clone)]
pub struct ConditionalExpr {
    pub span: Span,
    pub test: ExprRef,
    pub consequent: ExprRef,
    pub alternate: ExprRef,
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowAssign,
    LShiftAssign,
    RShiftAssign,
    RShift3Assign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    AndAssign,
    OrAssign,
    NullishAssign,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub span: Span,
    pub op: AssignOp,
    pub left: ExprRef,
    pub right: ExprRef,
}

#[derive(Debug, Clone)]
pub struct SequenceExpr {
    pub span: Span,
    pub expressions: Vec<ExprRef>,
}

#[derive(Debug, Clone)]
pub enum UpdateOp {
    PlusPlus,
    MinusMinus,
}

#[derive(Debug, Clone)]
pub struct UpdateExpr {
    pub span: Span,
    pub op: UpdateOp,
    pub arg: ExprRef,
    pub prefix: bool,
}

#[derive(Debug, Clone)]
pub struct AwaitExpr {
    pub span: Span,
    pub arg: ExprRef,
}

#[derive(Debug, Clone)]
pub struct YieldExpr {
    pub span: Span,
    pub arg: Option<ExprRef>,
    pub delegate: bool,
}

#[derive(Debug, Clone)]
pub struct SpreadExpr {
    pub span: Span,
    pub arg: ExprRef,
}

#[derive(Debug, Clone)]
pub struct TaggedTemplateExpr {
    pub span: Span,
    pub tag: ExprRef,
    pub template: TemplateLit,
}

#[derive(Debug, Clone)]
pub struct MetaPropExpr {
    pub span: Span,
    pub meta: String,
    pub property: String,
}

#[derive(Debug, Clone)]
pub struct ImportExpr {
    pub span: Span,
    pub source: ExprRef,
}

#[derive(Debug, Clone)]
pub struct JSXElement {
    pub span: Span,
    pub opening: JSXOpening,
    pub children: Vec<JSXChild>,
    pub closing: Option<JSXClosing>,
}

#[derive(Debug, Clone)]
pub struct JSXOpening {
    pub span: Span,
    pub name: JSXName,
    pub attrs: Vec<JSXAttr>,
    pub self_closing: bool,
}

#[derive(Debug, Clone)]
pub struct JSXClosing {
    pub span: Span,
    pub name: JSXName,
}

#[derive(Debug, Clone)]
pub enum JSXName {
    Ident(JSXIdent),
    Member(JSXMember),
    Namespace(JSXNamespace),
}

#[derive(Debug, Clone)]
pub struct JSXIdent {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct JSXMember {
    pub span: Span,
    pub object: Box<JSXName>,
    pub property: JSXIdent,
}

#[derive(Debug, Clone)]
pub struct JSXNamespace {
    pub span: Span,
    pub namespace: JSXIdent,
    pub name: JSXIdent,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum JSXAttr {
    Attr(JSXAttribute),
    Spread(SpreadExpr),
}

#[derive(Debug, Clone)]
pub struct JSXAttribute {
    pub span: Span,
    pub name: JSXName,
    pub value: Option<JSXAttrVal>,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum JSXAttrVal {
    Str(StrLit),
    Expr(ExprRef),
    Element(JSXElement),
    Fragment(JSXFragment),
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum JSXChild {
    Element(JSXElement),
    Fragment(JSXFragment),
    Expr(ExprRef),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct JSXFragment {
    pub span: Span,
    pub children: Vec<JSXChild>,
}

#[derive(Debug, Clone)]
pub struct TSAsExpr {
    pub span: Span,
    pub expr: ExprRef,
    pub type_ann: Box<TypeAnn>,
}

#[derive(Debug, Clone)]
pub struct TSSatisfiesExpr {
    pub span: Span,
    pub expr: ExprRef,
    pub type_ann: Box<TypeAnn>,
}

#[derive(Debug, Clone)]
pub struct TSTypeAssertionExpr {
    pub span: Span,
    pub expr: ExprRef,
    pub type_ann: Box<TypeAnn>,
}

#[derive(Debug, Clone)]
pub struct TSNonNullExpr {
    pub span: Span,
    pub expr: ExprRef,
}

#[derive(Debug, Clone)]
pub struct TSInstantiationExpr {
    pub span: Span,
    pub expr: ExprRef,
    pub type_params: Vec<TypeAnn>,
}

#[derive(Debug, Clone)]
pub struct ParenthesizedExpr {
    pub span: Span,
    pub expr: ExprRef,
}

#[derive(Debug, Clone)]
pub struct ChainExpr {
    pub span: Span,
    pub expr: ExprRef,
}

#[derive(Debug, Clone)]
pub struct InvalidExpr {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PrivateNameExpr {
    pub span: Span,
    pub name: Ident,
}

#[derive(Debug, Clone)]
pub struct RecordExpr {
    pub span: Span,
    pub props: Vec<ObjProp>,
}

#[derive(Debug, Clone)]
pub struct TupleExpr {
    pub span: Span,
    pub elements: Vec<Option<ExprRef>>,
}

#[derive(Debug, Clone)]
pub struct PipelineExpr {
    pub span: Span,
    pub input: ExprRef,
    pub body: ExprRef,
}

#[derive(Debug, Clone)]
pub enum TypeAnn {
    Any(Span),
    String(Span),
    Number(Span),
    Boolean(Span),
    Void(Span),
    Never(Span),
    Unknown(Span),
    Null(Span),
    Undefined(Span),
    Object(Span),
    Symbol(Span),
    BigInt(Span),
    Ident(Ident),
    Array(Box<TypeAnn>),
    Union(Vec<TypeAnn>),
    Intersection(Vec<TypeAnn>),
    Fn(Vec<TypeAnn>, Box<TypeAnn>),
    Lit(StrLit),
    LitNum(NumLit),
    LitBool(BoolLit),
    Generic(Ident, Vec<TypeAnn>),
    Tuple(Vec<TypeAnn>),
    Rest(Box<TypeAnn>),
    Optional(Box<TypeAnn>),
    Readonly(Box<TypeAnn>),
    KeyOf(Box<TypeAnn>),
    Typeof(Ident),
    Infer(Ident),
    Member(Box<TypeAnn>, Ident),
    Paren(Box<TypeAnn>),
    Mapped(Ident, Box<TypeAnn>),
    Conditional(Box<TypeAnn>, Box<TypeAnn>, Box<TypeAnn>, Box<TypeAnn>),
    This(Span),
    Pred(String, Box<TypeAnn>),
    Indexed(Box<TypeAnn>, Box<TypeAnn>),
    TsNull(Span),
}

impl AstNode for Ident {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ThisExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for SuperExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ArrayExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ObjectExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for KeyValueProp {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for MethodProp {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for GetterProp {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for SetterProp {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for FnExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ArrowExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ClassExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for MethodDef {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ClassProp {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for CtorDef {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for StaticBlock {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsIndexSig {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for NewExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for CallExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for OptionalCallExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for MemberExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for OptionalMemberExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for UnaryExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for UnaryOpExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for BinaryExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for LogicalExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ConditionalExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for AssignmentExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for SequenceExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for UpdateExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for AwaitExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for YieldExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for SpreadExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TaggedTemplateExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for MetaPropExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ImportExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXElement {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXOpening {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXClosing {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXIdent {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXMember {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXNamespace {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXAttribute {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for JSXFragment {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TSAsExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TSSatisfiesExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TSTypeAssertionExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TSNonNullExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TSInstantiationExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ParenthesizedExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for PrivateNameExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ChainExpr {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for InvalidExpr {
    fn span(&self) -> Span {
        self.span
    }
}
