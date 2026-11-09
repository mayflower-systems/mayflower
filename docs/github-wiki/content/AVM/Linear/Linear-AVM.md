<!--
title: Linear AVM
summary:
  A concrete AVM characteristic with linear segments and exact algebraic examples.
tags:
  - linear-avm
-->

The linear AVM uses the characteristic $g(x)=mx$, with fixed $m>0$, in the
[general AVM](Amplifying-Value-Machine). Its main and ramp are straight lines,
so prices, trade integrals, and floor-raise candidates can be expressed
algebraically.

Main has slope $m$. The ramp has slope $a=km$ for fixed $k>1$. Floor price,
ramp geometry, main placement, pricing progress, and circulating supply make up
the changing market state. Progress and circulation are distinct, just as in
the general model.

## Read the linear specification

| Page                                            | Contents                                                                    |
| ----------------------------------------------- | --------------------------------------------------------------------------- |
| [Specification](Linear-AVM-Specification)       | Symbols, pricing, backing, trade formulas, and state constraints            |
| [Evolution](Linear-AVM-Evolution)               | Exact updates for purchases, redemptions, contraction, and floor operations |
| [Floor raising](Linear-AVM-Floor-Raising)       | Constant-backing solutions and the state-dependent maximum floor            |
| [Floor investment](Linear-AVM-Floor-Investment) | Surplus-funded geometry and a complete numerical example                    |
| [Corollaries](Linear-AVM-Corollaries)           | Price-impact and capital relationships with their assumptions               |

## Why use a linear characteristic

Linear integrals make the accounting easy to inspect and provide useful worked
examples for the general design. They also make the tradeoff between main and
ramp price sensitivity explicit. The slope choices are market parameters, not
guarantees of returns, demand, or future floor growth.

The same [optional AVM features](AVM-Features) apply. Their custody, funding,
and authorization requirements remain necessary even when the underlying
arithmetic has a closed form.
