# rPython grammar (EBNF sketch)

Whitespace-sensitive: `NEWLINE` / `INDENT` / `DEDENT` come from the lexer. After `:`, the parser expects `NEWLINE INDENT` for a block.

```ebnf
module        ::= item* ;
item          ::= attrs? 'pub'? ( function | class | struct | enum | trait | impl | import ) ;
function      ::= 'def' ident generics? '(' params? ')' ( '->' type )? ':' block ;
class         ::= 'class' ident generics? ( '(' type ( ',' type )* ')' )? ':' item_block ;
struct        ::= 'struct' ident generics? ':' NEWLINE INDENT field+ DEDENT ;
field         ::= ident ':' type ;
enum          ::= 'enum' ident generics? ':' NEWLINE INDENT variant+ DEDENT ;
variant       ::= ident ( '(' type ( ',' type )* ')' )? ;
trait         ::= 'trait' ident generics? ':' item_block ;
impl          ::= 'impl' generics? ( path 'for' type | type ) ':' item_block ;
import        ::= 'import' path ( 'as' ident )? ;
import_from   ::= 'from' path 'import' ident ( 'as' ident )? ;

block         ::= NEWLINE INDENT stmt+ DEDENT | simple_stmt ;
stmt          ::= 'pass' | 'break' | 'continue' | 'return' expr?
              | 'if' expr ':' block ( 'elif' expr ':' block )* ( 'else' ':' block )?
              | 'while' expr ':' block
              | 'for' pat 'in' expr ':' block
              | 'match' expr ':' NEWLINE INDENT arm+ DEDENT
              | expr ( '=' expr )? ;
arm           ::= pat '=>' block ;

expr          ::= pratt_expr ;   /* or, and, compare, +-, */, unary, postfix */
postfix       ::= call | method | field | index | struct_lit ;
struct_lit    ::= '{' ( ident ':' expr ( ',' ident ':' expr )* )? '}'
              | path '{' ... '}' ;

type          ::= '&' 'mut'? type | '(' type ( ',' type )* ')' | path generics? ;
generics      ::= '[' ident ( ',' ident )* ']' ;
path          ::= ident ( '.' ident )* ;
```

Operator precedence (low → high): `or` < `and` < comparisons < `+` `-` < `*` `/` `//` `%` < unary < postfix.
