# Guests are user rows

A guest who plays the daily gets a real `user` row with a generated login and no credentials, and
signing up attaches credentials to that same row, so every game keeps a non-null owner and the
guest's daily survives the sign-up.

**Considered options**: games with no owner, claimed at sign-up (a nullable `user_id` everywhere
and a claim token to prove ownership); games kept in the browser only (impossible, the layout must
stay on the server).

**Consequences**: guest rows that never sign up pile up and need a cleanup job one day; a guest who
signs in to an existing account instead loses the guest's games.
