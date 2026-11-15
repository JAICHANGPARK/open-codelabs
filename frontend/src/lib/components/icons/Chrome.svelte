<script lang="ts">
	let {
		size = 24,
		strokeWidth = 2,
		class: className = ""
	}: {
		size?: number | string;
		strokeWidth?: number | string;
		class?: string;
	} = $props();
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width={strokeWidth}
	stroke-linecap="round"
	stroke-linejoin="round"
	class={className}
	aria-hidden="true"
>
	<circle cx="12" cy="12" r="10" />
	<circle cx="12" cy="12" r="4" />
	<line x1="21.17" x2="12" y1="8" y2="8" />
	<line x1="3.95" x2="8.54" y1="6.06" y2="14" />
	<line x1="10.88" x2="15.46" y1="21.94" y2="14" />
</svg>
