/// (a)-[b]->(c)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<'q, 'src> {
    pub init: &'q Entity<'q, 'src>,
    pub edges: &'q [&'q Edge<'q, 'src>],
}

/// -[a]->(b)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge<'q, 'src> {
    pub relation: &'q Relation<'src>,
    pub entity: &'q Entity<'q, 'src>,
}

/// -[a]->
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation<'src> {
    pub variable: &'src str,
    pub dir: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    /// -[]-
    None,
    /// <-[]-
    Left,
    /// -[]->
    Right,
}

/// (a)
/// (a:Person)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity<'q, 'src> {
    pub variable: Option<&'src str>,
    pub labels: &'q [&'src str],
}
