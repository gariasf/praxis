# Reverse-Z depth with an infinite far plane

The view distance reaches 2 km (ADR 0013). With a 0.1 m near plane and `Depth32Float`, standard depth resolves 2.4 m at 2 km; reversed depth with an infinite far plane resolves 0.15 mm. Depth is cleared to 0 and compared with `Greater`, in `Depth32Float`, with no far plane; shadow comparisons follow the same direction, and a Hi-Z pyramid keeps the minimum depth, which is the farthest. Accepted 2026-10-12. Planned; today depth clears to 1, compares with `Less`, and the far plane is 100 m.
