<!--
title: AVM Features
summary:
  Optional features built on the AVM pricing and backing rules.
tags:
  - avm-features
-->

These features build on the [AVM](Amplifying-Value-Machine). An application can
choose which operations to expose and how to authorize them. Each operation must
preserve the [pricing, backing, and settlement rules](AVM-Specification).

| Feature                                          | Purpose                                                                 |
| ------------------------------------------------ | ----------------------------------------------------------------------- |
| [Cash advances](Cash-Advance-Overview)           | Release reserve against the floor value of an encumbered AVA position   |
| [Cash advance mechanics](Cash-Advance-Mechanics) | Define capacity, collateral, fees, repayment, and net settlement        |
| [Call options](Call-Options)                     | Authorize issuance at the current floor without moving pricing progress |
| [Floor investment](Floor-Investment)             | Increase the floor using existing reserve surplus                       |

The core market's ordinary issuance, redemption, contraction, and area-preserving
floor raising do not require an option-grant mechanism or an advance facility.
