# Risks

- Policy bypass if direct addressing used → enforce all requests via controller; disallow peer-to-peer without controller auth.
- Infinite loops → add hopCount/loopGuard field; drop if >3.
