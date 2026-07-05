// In-memory session grant store. Grants bypass the matcher confirm branch.
// Lost on session restart (no persistence by design).

export class GrantStore {
	private readonly granted = new Set<string>();

	grant(name: string): boolean {
		this.granted.add(name);
		return true;
	}

	revoke(name: string): boolean {
		return this.granted.delete(name);
	}

	has(name: string): boolean {
		return this.granted.has(name);
	}

	list(): string[] {
		return [...this.granted].sort();
	}

	clear(): void {
		this.granted.clear();
	}
}