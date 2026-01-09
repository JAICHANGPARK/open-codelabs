<script lang="ts">
    import { onMount } from "svelte";

    interface Props {
        glitchColors?: string[];
        glitchSpeed?: number;
        centerVignette?: boolean;
        outerVignette?: boolean;
        smooth?: boolean;
        characters?: string;
    }

    let {
        glitchColors = ["#2b4539", "#61dca3", "#61b3dc"],
        glitchSpeed = 50,
        centerVignette = false,
        outerVignette = true,
        smooth = true,
        characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$&*()-_+=/[]{};:<>.,0123456789",
    }: Props = $props();

    let canvas: HTMLCanvasElement | null = $state(null);
    let animationFrame: number | null = null;
    let context: CanvasRenderingContext2D | null = null;

    let letters: {
        char: string;
        color: string;
        targetColor: string;
        colorProgress: number;
    }[] = [];
    let grid = { columns: 0, rows: 0 };
    let lastGlitchTime = Date.now();

    const lettersAndSymbols = characters.split("");
    const fontSize = 16;
    const charWidth = 10;
    const charHeight = 20;

    function getRandomChar() {
        return lettersAndSymbols[
            Math.floor(Math.random() * lettersAndSymbols.length)
        ];
    }

    function getRandomColor() {
        return glitchColors[Math.floor(Math.random() * glitchColors.length)];
    }

    function hexToRgb(hex: string) {
        const shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
        hex = hex.replace(shorthandRegex, (_m, r, g, b) => {
            return r + r + g + g + b + b;
        });

        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result
            ? {
                  r: parseInt(result[1], 16),
                  g: parseInt(result[2], 16),
                  b: parseInt(result[3], 16),
              }
            : null;
    }

    function interpolateColor(
        start: { r: number; g: number; b: number },
        end: { r: number; g: number; b: number },
        factor: number,
    ) {
        const result = {
            r: Math.round(start.r + (end.r - start.r) * factor),
            g: Math.round(start.g + (end.g - start.g) * factor),
            b: Math.round(start.b + (end.b - start.b) * factor),
        };
        return `rgb(${result.r}, ${result.g}, ${result.b})`;
    }

    function calculateGrid(width: number, height: number) {
        const columns = Math.ceil(width / charWidth);
        const rows = Math.ceil(height / charHeight);
        return { columns, rows };
    }

    function initializeLetters(columns: number, rows: number) {
        grid = { columns, rows };
        const totalLetters = columns * rows;
        letters = Array.from({ length: totalLetters }, () => ({
            char: getRandomChar(),
            color: getRandomColor(),
            targetColor: getRandomColor(),
            colorProgress: 1,
        }));
    }

    function drawLetters() {
        if (!context || !canvas || letters.length === 0) return;
        const rect = canvas.getBoundingClientRect();
        context.clearRect(0, 0, rect.width, rect.height);
        context.font = `${fontSize}px monospace`;
        context.textBaseline = "top";

        letters.forEach((letter, index) => {
            const x = (index % grid.columns) * charWidth;
            const y = Math.floor(index / grid.columns) * charHeight;
            context!.fillStyle = letter.color;
            context!.fillText(letter.char, x, y);
        });
    }

    function updateLetters() {
        if (letters.length === 0) return;

        const updateCount = Math.max(1, Math.floor(letters.length * 0.05));

        for (let i = 0; i < updateCount; i++) {
            const index = Math.floor(Math.random() * letters.length);
            if (!letters[index]) continue;

            letters[index].char = getRandomChar();
            letters[index].targetColor = getRandomColor();

            if (!smooth) {
                letters[index].color = letters[index].targetColor;
                letters[index].colorProgress = 1;
            } else {
                letters[index].colorProgress = 0;
            }
        }
    }

    function handleSmoothTransitions() {
        let needsRedraw = false;
        letters.forEach((letter) => {
            if (letter.colorProgress < 1) {
                letter.colorProgress += 0.05;
                if (letter.colorProgress > 1) letter.colorProgress = 1;

                const startRgb = hexToRgb(letter.color);
                const endRgb = hexToRgb(letter.targetColor);
                if (startRgb && endRgb) {
                    letter.color = interpolateColor(
                        startRgb,
                        endRgb,
                        letter.colorProgress,
                    );
                    needsRedraw = true;
                }
            }
        });

        if (needsRedraw) {
            drawLetters();
        }
    }

    function animate() {
        const now = Date.now();
        if (now - lastGlitchTime >= glitchSpeed) {
            updateLetters();
            drawLetters();
            lastGlitchTime = now;
        }

        if (smooth) {
            handleSmoothTransitions();
        }

        animationFrame = requestAnimationFrame(animate);
    }

    function resize() {
        if (!canvas) return;
        const parent = canvas.parentElement;
        if (!parent) return;

        const dpr = window.devicePixelRatio || 1;
        const rect = parent.getBoundingClientRect();

        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;

        canvas.style.width = `${rect.width}px`;
        canvas.style.height = `${rect.height}px`;

        if (context) {
            context.setTransform(dpr, 0, 0, dpr, 0, 0);
        }

        const { columns, rows } = calculateGrid(rect.width, rect.height);
        initializeLetters(columns, rows);
        drawLetters();
    }

    onMount(() => {
        if (!canvas) return;
        context = canvas.getContext("2d");
        resize();
        animate();

        let resizeTimeout: ReturnType<typeof setTimeout>;
        const handleResize = () => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                if (animationFrame) cancelAnimationFrame(animationFrame);
                resize();
                animate();
            }, 100);
        };

        window.addEventListener("resize", handleResize);

        return () => {
            if (animationFrame) cancelAnimationFrame(animationFrame);
            window.removeEventListener("resize", handleResize);
        };
    });

    // Re-animate when glitchSpeed or smooth props change
    $effect(() => {
        if (animationFrame) cancelAnimationFrame(animationFrame);
        animate();
    });
</script>

<div class="relative w-full h-full bg-black overflow-hidden">
    <canvas bind:this={canvas} class="block w-full h-full"></canvas>

    {#if outerVignette}
        <div
            class="absolute inset-0 pointer-events-none"
            style="background: radial-gradient(circle, rgba(0,0,0,0) 60%, rgba(0,0,0,1) 100%)"
        ></div>
    {/if}

    {#if centerVignette}
        <div
            class="absolute inset-0 pointer-events-none"
            style="background: radial-gradient(circle, rgba(0,0,0,0.8) 0%, rgba(0,0,0,0) 60%)"
        ></div>
    {/if}
</div>
