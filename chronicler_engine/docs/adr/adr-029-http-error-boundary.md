# ADR-029: HttpError Trait Boundary — Keep Axum Out of Application Layer

**Date:** 2026-07-06
**Status:** Accepted

## Context

The application layer currently holds `impl IntoResponse for ApplicationError` in `src/application/application_service.rs` (lines 9-10 import `axum::response::IntoResponse`; the impl block is at line 95). This import drags an HTTP-framework type into the application module, which violates the dependency invariant established by ADR-027: the core (`domain/`, `application/`) depends on port traits only and must not import adapter types directly.

The holistic hexagon investigation (2026-07-05) flagged this as a hexagonal-boundary violation in the B-series / C-series findings, and the H0 super-plan track recommends formalising the boundary with a small port trait owned by the application layer and an HTTP-mapping impl owned by the adapter.

The current coupling has three concrete costs:

1. The application layer cannot be compiled or tested without the axum crate in scope.
2. Switching HTTP frameworks (axum → actix, warp, hyper-router, etc.) requires editing application-layer code, not just adapter code.
3. Any HTTP concern (status codes, error body shape, content negotiation) leaks into application-layer types via the `IntoResponse` impl.

A port trait (`HttpError`) owned by the application layer inverts the direction: the application declares what it needs from an HTTP mapper, and the adapter provides the mapper. This is the same pattern already used for `LlmProvider` (application port, adapter impls).

## Decision

Introduce a port trait `HttpError` owned by the application layer, and move the HTTP-mapping impl out of the application layer into the HTTP driving adapter.

### Trait Location and Shape

- Trait `HttpError` lives in `src/application/error.rs` (application layer).
- Trait methods commit to this shape (exact names may be refined at impl time; the ADR locks the boundary, not the spelling):
  - `fn status_code() -> StatusCode`
  - `fn error_body() -> ErrorResponse`
- The trait depends on types `StatusCode` and `ErrorResponse` that are themselves defined in or below the application layer — not on axum types.

### Impl Location

- `impl IntoResponse for ApplicationError` lives in `src/adapters/driving/http/error.rs` (HTTP driving adapter).
- The adapter impl calls the `HttpError` trait methods to obtain the `StatusCode` and `ErrorResponse`, then constructs the axum response.
- The old impl in `src/application/application_service.rs` is deleted (this is H1 work, not H0; H0 only locks the decision).

### Explicit Impl, Not Blanket

- The HTTP mapping is implemented as a single explicit `impl IntoResponse for ApplicationError`.
- Blanket impls (`impl<T: HttpError> IntoResponse for T`) are **not** introduced for this case.
- A blanket impl becomes appropriate only when at least two distinct application-layer error types need the same HTTP mapping.

### Dependency Invariant (Reinforced)

- Application layer imports zero axum types directly.
- All axum-coupled code paths go through traits whose trait-defining types live at or below the application layer.
- The HTTP adapter remains the only module that imports both `HttpError` and `axum::response::IntoResponse`.

### Why Explicit Impl Over Blanket Impl

- Currently exactly one caller (`ApplicationError`) needs HTTP mapping. Blanket impls have zero current call sites and would expand the impl surface for hypothetical future types.
- A blanket `impl<T: HttpError> IntoResponse for T` would let any new type that implements `HttpError` automatically inherit the application's HTTP mapping, including types whose authors did not intend HTTP exposure.
- Explicit impls are boring-by-default: adding a new mappable error type requires a deliberate `impl IntoResponse for NewType` block in the adapter, which is exactly the friction that catches accidental coupling.
- YAGNI: the blanket pattern is reserved for the day a second type needs identical mapping. That day has not arrived.

## Consequences

### Positive

- Application layer has zero axum dependency. `application/error.rs` and `application/application_service.rs` compile without the axum crate in scope.
- Application error types are testable in isolation — no axum types in error unit tests, no risk of breaking HTTP assumptions when changing application logic.
- Hexagonal dependency invariant (ADR-027) is respected at this boundary. The port-trait pattern matches `LlmProvider` precedent.
- The HTTP-mapping concern (status code selection, body shape, content-type negotiation) is owned entirely by the HTTP adapter, where it belongs.
- Future HTTP-framework swap is a single-adapter change.

### Negative

- The trait introduces one indirection layer for HTTP error mapping. Readers tracing an error response must follow the trait method into the adapter impl.
- A second error type needing the same HTTP mapping in the future requires a new explicit `impl IntoResponse for NewType` block in the adapter (small, deliberate friction by design).
- Method names and the `ErrorResponse` type shape are not yet pinned by this ADR — they are refined at impl time. Slight risk of churn if names are picked badly; mitigated by the fact that this is application-internal API, not public.

### Trade-offs

- Chose explicit impl over blanket impl for boring-by-default safety — the blanket pattern's convenience is not worth the loss of explicitness at this scale.
- Chose port-trait pattern (trait in application, impl in adapter) over alternative patterns (e.g. free function `to_http_response(err) -> axum::Response` in the adapter taking `&ApplicationError`) because the trait preserves the call-site form `err.into_response()` and lets future error types opt in individually.
- Chose to keep `StatusCode` and `ErrorResponse` as application-layer types rather than re-exporting axum's `StatusCode` because the application must not name axum types; the adapter maps from the application type to the axum type at the boundary.
- Accepted that this ADR locks the boundary and the high-level shape, not the exact method names or the precise body of `ErrorResponse`. Locking the spelling would be premature before H1 implementation reveals the ergonomic details.

## Related ADRs

- ADR-027: Hexagonal Architecture Migration — parent decision; defines the dependency invariant that this ADR enforces at the HTTP-error boundary.
- ADR-018: Application Service (planned, not yet drafted) — the application service module is the producer of the error type whose HTTP mapping is being relocated. Coordination expected at H1 implementation time.

## History

- **2026-07-06**: Initial decision. Locks the HttpError trait boundary, the IntoResponse impl location, and the explicit-impl-over-blanket choice.
