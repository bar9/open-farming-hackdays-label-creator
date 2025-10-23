Architecture
- [x] Country lists are duplicated and inconsistent: Single component with CH, EU, Keine Herkunftsangabe nötig, ALL ISO countries
- [x] is_meat, is_fish, is_beef etc: things derived from categories should be handled in a centralised service
- [x] debug all rules to browser console: set of currently active rules? match? result?

UI
- [x] multiple validation errors: stack them instead of just showing one
- [x] umstellung additional texts should be just print text (black on white, no borders or anything, just to the left of the logo)
- [x] check spacing in the ingredient properties: the dist between Zusammengesetzte Zutat and Namensgebende Zutat is ok, the others are too close

Rules
- [x] check for logic that is strongly tied to domain logic but are not represented as rules yet, and transform them into rules if its feasible and low risk

General
- [x] look for refactoring opportunities that are obvious and low risk
- [x] look for test gaps
- [x] look for hard coded strings and move them to the usual yaml files