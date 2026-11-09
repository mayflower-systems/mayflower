<!--
title: AVM Rationale
summary:
  Why the AVM combines buyer-funded reserves, price discovery, and an evolving floor.
tags:
  - avm
  - pricing
-->

The [AVM](Amplifying-Value-Machine) combines a deterministic market with a
reserve-denominated minimum redemption value. This page explains the design's
purpose and tradeoffs. Exact rules belong to the [specification](AVM-Specification).

## Buyers fund the reserve

Purchasing AVA supplies the capital backing it. Redemption returns capital from
that reserve according to the pricing schedule. The core market therefore has
no separate liquidity-provider position whose withdrawal removes its backing.

The reserve is still finite. A marginal quote does not mean every circulating
token can be sold at that quote, and backing held in collateralized advances
is not immediately liquid. These distinctions are why the system tracks required
backing, liquid reserve, and advances separately.

## A floor separates backing from price discovery

The floor assigns a minimum gross reserve claim to unencumbered AVA. The
schedule above the floor supplies the price-discovery component. A purchase
above the floor retains exposure to that premium falling, even though the floor
itself cannot decrease under the permitted transitions.

An area-preserving floor raise reallocates existing backing: it increases the
floor component while reducing the value assigned above the floor. It creates
no new reserve. Surplus-funded investment can instead increase required backing
using capital already available to the market.

This is useful when a market wants part of accumulated backing to support a
persistent minimum claim. It is not a promise of investment returns or a floor
measured in any currency other than the reserve asset.

## The ramp gives the schedule room to change

The ramp lies below the backward extension of main. Advancing its endpoint can
replace main area with less expensive ramp area, making room for a higher floor.
That geometric relationship is the basis of [floor raising](AVM-Floor-Raising).

A sale into the ramp reattaches main at the remaining progress. The next purchase
then follows main from that point. Ordinary purchases do not climb an old ramp:
the ramp describes retained backing and redemption behavior. Contraction may
allow later raises at lower progress, but no trading pattern guarantees future
floor growth.

## Separate progress from circulation

An authorized floor-priced issuance adds AVA and its floor backing without
moving the live price. Tracking circulation separately from pricing progress
expresses this directly. The same distinction supports a bounded floor-redemption
operation without moving the schedule.

It also prevents a common accounting mistake: when those quantities differ,
required backing is not simply the area from zero to current circulation.

## Advances use the same backing

A [cash advance](Cash-Advance-Overview) releases reserve against an AVA position's
floor value. Its principal has no time-based interest increase in this design,
and a fixed collateral quantity with a nondecreasing floor retains its advance
capacity. The application must keep collateral encumbered and check liquid
reserve before paying a draw.

This avoids a separate lender pool, while making collateral custody and reserve
liquidity essential. Releasing reserve now also consumes part of the holder's
future unencumbered redemption entitlement.
