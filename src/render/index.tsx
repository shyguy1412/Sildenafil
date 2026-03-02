import './index.css';
import 'normalize.css';
import { h, render } from 'preact';
import { HelloWorld } from './components/HelloWorld';
import type { Event as CoreEvent, EventVariants } from '@core';

type Props = {};

const coreEventTarget = new EventTarget();
export const EventSystem = {
    addEventListener<T extends keyof EventVariants>(
        event: T,
        callback: (event: CustomEvent<CoreEvent<T>>) => void,
        options?: AddEventListenerOptions | boolean,
    ): void {
        coreEventTarget.addEventListener(event, callback as EventListener, options);
    },
    dispatchEvent<T extends keyof EventVariants>(
        event: CustomEvent<CoreEvent<T>>,
    ): boolean {
        return coreEventTarget.dispatchEvent(event);
    },
    removeEventListener(
        event: string,
        callback: EventListenerOrEventListenerObject | null,
        options?: EventListenerOptions | boolean,
    ): void {
        coreEventTarget.removeEventListener(event, callback, options);
    },
};

EventSystem.addEventListener('CommunityGoal', ({ detail: data }) => console.log(data));

function Index({}: Props) {
    return <HelloWorld></HelloWorld>;
}
__module_bridge_init.then(async () => {
    core.setEventListener((event, data) => {
        EventSystem.dispatchEvent(new CustomEvent(event, { detail: data }));
    });

    const graphics = await core.getGraphicsConfig();
    const parser = new DOMParser();
    const xml = parser.parseFromString(graphics, 'text/xml');

    const matrixRed = xml.querySelector('GUIColour>Default>MatrixRed')!.textContent;
    const matrixGreen = xml.querySelector('GUIColour>Default>MatrixGreen')!.textContent;
    const matrixBlue = xml.querySelector('GUIColour>Default>MatrixBlue')!.textContent;

    const data = [
        { col: 'red', value: matrixRed.split(',') },
        { col: 'green', value: matrixGreen.split(',') },
        { col: 'blue', value: matrixBlue.split(',') },
    ];

    const testData = [
        { col: 'red', value: ['1', '0', '0'] },
        { col: 'green', value: ['0', '1', '0'] },
        { col: 'blue', value: ['0', '0', '1'] },
    ];


    const rgb = ['r', 'g', 'b'];
    for (const matrix of testData) {
        const col = matrix.col;
        for (const cur in rgb) {
            const attr = `data-${col}-matrix-${rgb[cur]}`;
            document.body.parentElement!.setAttribute(attr, ""+(+matrix.value[cur]));
        }
    }

    render(<Index></Index>, document.body);
});
