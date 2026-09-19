---
hi: 1
families: [LINK]
---

# Contracts across repositories

## Intent

Modules depend on modules in other repositories, and a contract should be able to say so. Naming that dependency must stay free: an ordinary check never reaches the network, so nobody pays for the declaration on every build. Confirming those references are real is a thing I ask for deliberately, and there is nothing central to register with.

## Criteria

- **LINK-1**  A contract can declare that it depends on a module in another repository.
- **LINK-2**  Those cross-repository references never cost a network call during an ordinary check.
- **LINK-3**  When I do want to know the references are real, I ask for that explicitly.
  - **LINK-3.a**  I can go further and confirm the upstream names the reference relies on still exist.
- **LINK-4**  A repository can publish what it offers other repositories in one small file.
  - **LINK-4.a**  That file is generated from the contracts that already exist rather than maintained by hand.
- **LINK-5**  There is no central service to register with.
  - **LINK-5.a**  There is no lock file to keep up to date.
