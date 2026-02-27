import './index.css';
import 'normalize.css';
import { h, render } from "preact";
import { HelloWorld } from "./components/HelloWorld";
import type { EventVariants, Event as CoreEvent } from '@core';

type Props = {};

const coreEventTarget = new EventTarget();
export const EventSystem = {
    addEventListener<T extends keyof EventVariants>(event: T, callback: (event: CustomEvent<CoreEvent<T>>) => void, options?: AddEventListenerOptions | boolean): void {
        coreEventTarget.addEventListener(event, callback as EventListener, options);
    },
    dispatchEvent<T extends keyof EventVariants>(event: CustomEvent<CoreEvent<T>>): boolean {
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
    console.log(core);
    core.setEventListener((event, data) => {
        EventSystem.dispatchEvent(new CustomEvent(event, { detail: data }));
    });
    render(<Index></Index>, document.body);
});

