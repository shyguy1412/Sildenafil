import './index.css';
import 'normalize.css';
import { h, render } from "preact";
import { HelloWorld } from "./components/HelloWorld";
import type { Events } from '@core';

type Props = {};

const coreEventTarget = new EventTarget();
export const EventSystem = {
    addEventListener<T extends keyof Events>(event: T, callback: (event: CustomEvent<Events[T]>) => void, options?: AddEventListenerOptions | boolean): void {
        coreEventTarget.addEventListener(event, callback as EventListener, options);
    },
    dispatchEvent(event: CustomEvent<Events[keyof Events]>): boolean {
        return coreEventTarget.dispatchEvent(event);
    },
    removeEventListener(event: string, callback: EventListenerOrEventListenerObject | null, options?: EventListenerOptions | boolean): void {
        coreEventTarget.removeEventListener(event, callback, options);
    }
};

EventSystem.addEventListener("CommunityGoal", ({ detail: data }) => console.log(data));

function Index({ }: Props) {

    return <HelloWorld></HelloWorld>;
};
__module_bridge_init.then(() => {
    core.setEventListener((event, ...args) => {
        console.log(event, args);
        // EventSystem.dispatchEvent(new CustomEvent(event, { detail: data }));
    });
    render(<Index></Index>, document.body);
});

