import * as ButtonModule from "./button.svelte";

type ButtonProps = import("./button.svelte").ButtonProps;
type ButtonSize = import("./button.svelte").ButtonSize;
type ButtonVariant = import("./button.svelte").ButtonVariant;

const Root = (ButtonModule as any).default;
const buttonVariants = (ButtonModule as any).buttonVariants;

export {
	Root,
	type ButtonProps as Props,
	//
	Root as Button,
	buttonVariants,
	type ButtonProps,
	type ButtonSize,
	type ButtonVariant,
};
