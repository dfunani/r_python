#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ErrorCode(pub u16);

impl ErrorCode {
    pub const E0001: Self = Self(1);
    pub const E0002: Self = Self(2);

    // Name resolution (E02xx)
    pub const E0201: Self = Self(201);
    pub const E0202: Self = Self(202);
    pub const E0203: Self = Self(203);
    pub const E0204: Self = Self(204);

    // Type checking (E03xx)
    pub const E0300: Self = Self(300);
    pub const E0301: Self = Self(301);
    pub const E0302: Self = Self(302);
    pub const E0303: Self = Self(303);
    pub const E0304: Self = Self(304);
    pub const E0305: Self = Self(305);
    pub const E0306: Self = Self(306);

    pub fn as_str(self) -> String {
        format!("E{:04}", self.0)
    }

    pub fn explain(self) -> &'static str {
        match self.0 {
            1 => "invalid character in source",
            2 => "unterminated string literal",
            201 => "duplicate definition in the same scope",
            202 => "used before definition",
            203 => "unresolved import",
            204 => "cannot resolve name",
            300 => "type mismatch",
            301 => "wrong number of arguments",
            302 => "cannot infer type",
            303 => "non-exhaustive match",
            304 => "wrong return type",
            305 => "ambiguous type or trait resolution",
            306 => "trait bound not satisfied",
            _ => "no documentation available for this error code",
        }
    }
}
