# Parser accepts extern blocks; full codegen wiring is P6+.
# extern "C" {
#     fn strlen(s: *const u8) -> int
# }

def main() -> int:
    print("extern blocks — see DESIGN_SPEC.md")
    return 0
