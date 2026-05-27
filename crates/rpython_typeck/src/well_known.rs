use rpython_ids::{DefId, TypeId};
use rpython_resolve::BuiltinDefs;
use rpython_types::TypeDatabase;

/// Builtin types and functions wired into the typechecker.
#[derive(Clone, Copy, Debug)]
pub struct WellKnown {
    pub int: TypeId,
    pub bool: TypeId,
    pub str: TypeId,
    pub unit: TypeId,
    pub float: TypeId,
    pub never: TypeId,
    pub print: DefId,
    pub ty_int_def: DefId,
    pub ty_bool_def: DefId,
    pub ty_str_def: DefId,
    pub ty_unit_def: DefId,
}

impl WellKnown {
    pub fn new(db: &mut TypeDatabase, builtins: &BuiltinDefs) -> Self {
        Self {
            int: db.int(),
            bool: db.bool(),
            str: db.str(),
            unit: db.unit(),
            float: db.float(),
            never: db.never(),
            print: builtins.print,
            ty_int_def: builtins.ty_int,
            ty_bool_def: builtins.ty_bool,
            ty_str_def: builtins.ty_str,
            ty_unit_def: builtins.ty_unit,
        }
    }

    pub fn type_for_builtin_def(&self, db: &TypeDatabase, def: DefId) -> Option<TypeId> {
        if def == self.ty_int_def {
            Some(self.int)
        } else if def == self.ty_bool_def {
            Some(self.bool)
        } else if def == self.ty_str_def {
            Some(self.str)
        } else if def == self.ty_unit_def {
            Some(self.unit)
        } else {
            None
        }
    }
}
