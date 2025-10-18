type Events = import("@core").Events;

declare global {
	const core: {
		on: <T extends keyof Events>(event: T, callback: (event: Events[T]) => void) => Promise<void>;
	};
}

export default global;
