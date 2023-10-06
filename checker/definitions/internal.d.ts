declare function debug_context(): void performs const debug_context;
declare function print_type(t: any): void performs const print_type;
declare function debug_type(t: any): void performs const debug_type;
declare function debug_effects(t: () => {}): void performs const debug_effects;
declare function is_dependent(t: any): void performs const is_dependent;

declare function context_id(): void performs const context_id;
declare function context_id_chain(): void performs const context_id_chain;

// A function, as it should be!
declare function satisfies<T>(t: T): T performs const satisfies;

declare function compile_type_to_object<T>(): any performs const compile_type_to_object;

interface Array<T> {
    length: number;

    [index: number]: T;

    push(item: any) performs {
        this[this.length] = item;
        return ++this.length
    }

    last() performs {
        return this[this.length - 1]
    }
}

interface Math {
    sin(x: number): number performs const sin;
    cos(x: number): number performs const cos;
    tan(x: number): number performs const tan;
    trunc(x: number): number performs const trunc;
    sqrt(x: number): number performs const sqrt;
    cbrt(x: number): number performs const cbrt;
}

interface string {
    toUpperCase(): string performs const uppercase;
    toLowerCase(): string performs const lowercase;
}

interface Console {
    log(msg: any): void;
}

interface JSON {
    // TODO any temp
    parse(input: string): any;

    // TODO any temp
    stringify(input: any): string;
}

declare var JSON: JSON;
declare var Math: Math;
declare var console: Console;

declare function JSXH(tagname: string, attributes: any, children: any) performs {
    return tagname
}
