# Fast enum conversion

This crate provides zerocost conversion between enums. An enum `Src` can be converted to another enum `Dest` when the following satisfies:

- For each variants of `Src`, counterparts exist in `Dest`.

Here, an variant ans its "counterpart" satisfies all of them:

- Both has the same tag names
- `Fields` of both variants are equal. For example, one has structural fields, another should have the same.
- All corresponding type of `Fields` are the same.

It performs zerocost conversion when following satisfies for all counterparts:

- The [`std::mem::Discriminant`] of them are equal.
- The fields has same offsets.
- Have consistent memory layouts.
