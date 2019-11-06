macro_rules! unpack {
    ($x: expr, $y: ident, $b: block) => {
        if let Some(ref $y) = $x {
            $b
        }
        else {
            unreachable!();
        }
    }
}