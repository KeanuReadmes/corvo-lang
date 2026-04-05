//! GNU-compatible version string ordering (`strverscmp`), ported from musl libc.

use std::cmp::Ordering;

/// Compare `a` and `b` using GNU `strverscmp` / musl semantics.
pub fn strverscmp(a: &str, b: &str) -> Ordering {
    let l = a.as_bytes();
    let r = b.as_bytes();
    let mut i: usize = 0;
    let mut dp: usize = 0;
    let mut z: i32 = 1;

    loop {
        let lc = l.get(i).copied();
        let rc = r.get(i).copied();
        match (lc, rc) {
            (Some(x), Some(y)) if x == y => {
                if x == 0 {
                    return Ordering::Equal;
                }
                if !x.is_ascii_digit() {
                    dp = i + 1;
                    z = 1;
                } else if x != b'0' {
                    z = 0;
                }
                i += 1;
            }
            (Some(_), Some(_)) => break,
            (None, None) => return Ordering::Equal,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
        }
    }

    let li = *l.get(i).unwrap_or(&0);
    let ri = *r.get(i).unwrap_or(&0);
    let ld = l.get(dp).copied().unwrap_or(0);
    let rd = r.get(dp).copied().unwrap_or(0);

    if ld.wrapping_sub(b'1') < 9 && rd.wrapping_sub(b'1') < 9 {
        let mut j = i;
        while l
            .get(j)
            .copied()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            if !r
                .get(j)
                .copied()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Ordering::Greater;
            }
            j += 1;
        }
        if r.get(j)
            .copied()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return Ordering::Less;
        }
    } else if z != 0 && dp < i && (li.is_ascii_digit() || ri.is_ascii_digit()) {
        return (li as i32 - b'0' as i32).cmp(&(ri as i32 - b'0' as i32));
    }

    (li as i32).cmp(&(ri as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_order_basic() {
        let mut v = vec!["a10", "a2", "a1"];
        v.sort_by(|x, y| strverscmp(x, y));
        assert_eq!(v, vec!["a1", "a2", "a10"]);
    }
}
