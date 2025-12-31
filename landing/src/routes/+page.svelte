<script lang="ts">
	import { onMount } from "svelte";
	import {
		Rocket,
		Shield,
		Cpu,
		Globe,
		Users,
		Zap,
		Github,
		ChevronRight,
		BookOpen,
		BarChart3,
		MessageSquare,
		Award,
		CheckCircle2,
	} from "lucide-svelte";

	let lang = $state("ko");
	let quickstartType = $state("docker"); // 'docker' or 'podman'

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				entries.forEach((entry) => {
					if (entry.isIntersecting) {
						entry.target.classList.add("reveal-visible");
					}
				});
			},
			{ threshold: 0.1 },
		);

		document
			.querySelectorAll(".reveal")
			.forEach((el) => observer.observe(el));
	});

	const content = $derived({
		nav: {
			features: lang === "ko" ? "주요 기능" : "Features",
			roles: lang === "ko" ? "사용자별 기능" : "Roles",
			quickstart: lang === "ko" ? "빠른 시작" : "Quickstart",
			getStarted: lang === "ko" ? "시작하기" : "Get Started",
		},
		hero: {
			badge:
				lang === "ko"
					? "Rust & Svelte 기반"
					: "Powered by Rust & Svelte",
			title1: lang === "ko" ? "더 스마트한" : "Interactive Workshops",
			title2: lang === "ko" ? "코딩 워크숍의 시작." : "Made Simple.",
			desc:
				lang === "ko"
					? "Open Codelabs는 현대적인 오픈소스 코딩 워크숍 플랫폼입니다. Google Codelabs의 경험을 AI와 함께 더욱 강력하게 만들었습니다."
					: "Open Codelabs is a modern, open-source platform for hosting hands-on coding workshops. Inspired by Google Codelabs, enhanced with AI.",
			ctaPrimary:
				lang === "ko" ? "5분 만에 시작하기" : "Get Started in 5m",
			ctaSecondary: lang === "ko" ? "문서 보기" : "View Docs",
		},
		quickstart: {
			title:
				lang === "ko"
					? "5분 만에 워크숍 준비 끝"
					: "Ready in 5 minutes",
			desc:
				lang === "ko"
					? "Docker 또는 Podman을 사용하여 복잡한 설정 없이 즉시 실행할 수 있습니다."
					: "Launch instantly with Docker or Podman, no complex setup required.",
			tabs: {
				docker: "Docker",
				podman: "Podman",
			},
			steps: [
				{
					title: lang === "ko" ? "저장소 클론" : "Clone Repository",
					code: "git clone https://github.com/jaichangpark/open-codelabs.git\ncd open-codelabs",
				},
				{
					title: lang === "ko" ? "컨테이너 실행" : "Run Container",
					code:
						quickstartType === "docker"
							? "docker compose up --build"
							: "podman-compose up --build",
				},
				{
					title: lang === "ko" ? "접속 및 관리" : "Access & Manage",
					desc:
						lang === "ko"
							? "http://localhost:5173 에 접속하여 관리를 시작하세요."
							: "Open http://localhost:5173 to start managing.",
				},
			],
		},
		features: {
			title:
				lang === "ko"
					? "워크숍에 필요한 모든 것"
					: "Everything you need for workshops",
			desc:
				lang === "ko"
					? "가장 유연하고 사용하기 쉬운 코드랩 플랫폼을 목표로 바닥부터 설계되었습니다."
					: "Built from the ground up to be the most flexible and easy-to-use codelab platform.",
			items: [
				{
					icon: Users,
					title:
						lang === "ko" ? "이원화된 시스템" : "Dual Role System",
					description:
						lang === "ko"
							? "퍼실리테이터와 참가자 모두에게 최적화된 경험을 제공합니다."
							: "Optimized experiences for both Facilitators and Attendees.",
				},
				{
					icon: Cpu,
					title:
						lang === "ko"
							? "AI 콘텐츠 생성"
							: "AI-Assisted Content",
					description:
						lang === "ko"
							? "Google Gemini AI를 사용하여 고품질의 코드랩을 즉시 생성할 수 있습니다."
							: "Generate high-quality codelab content instantly with AI assistance.",
				},
				{
					icon: Shield,
					title:
						lang === "ko" ? "안전하고 유연함" : "Secure & Flexible",
					description:
						lang === "ko"
							? "Rust와 SQLite를 통한 자체 호스팅 또는 Firebase 서버리스 배포가 가능합니다."
							: "Self-host with Rust and SQLite or go serverless with Firebase.",
				},
				{
					icon: Globe,
					title: lang === "ko" ? "오픈 소스" : "Open Source",
					description:
						lang === "ko"
							? "커뮤니티 주도형 프로젝트로 모든 소스가 공개되어 있으며 자유롭게 커스텀 가능합니다."
							: "Community-driven and transparent. Customize every aspect as you need.",
				},
			],
		},
		facilitator: {
			title:
				lang === "ko" ? "퍼실리테이터를 위한 기능" : "For Facilitators",
			features: [
				{
					icon: Zap,
					text:
						lang === "ko"
							? "AI를 통한 1분 만의 코드랩 초안 생성"
							: "Generate drafts in seconds with AI",
				},
				{
					icon: BarChart3,
					text:
						lang === "ko"
							? "실시간 참가자 진행률 대시보드"
							: "Real-time attendee progress dashboard",
				},
				{
					icon: MessageSquare,
					text:
						lang === "ko"
							? "참가자 피드백 실시간 수집 및 분석"
							: "Collect and analyze live feedback",
				},
				{
					icon: Shield,
					text:
						lang === "ko"
							? "공개/비공개 워크숍 관리 및 권한 설정"
							: "Manage public/private workshop visibility",
				},
			],
		},
		attendee: {
			title: lang === "ko" ? "참가자를 위한 기능" : "For Attendees",
			features: [
				{
					icon: BookOpen,
					text:
						lang === "ko"
							? "몰입감 있는 단계별 학습 인터페이스"
							: "Immersive step-by-step learning",
				},
				{
					icon: Zap,
					text:
						lang === "ko"
							? "코드 실행 결과 및 실시간 도움말"
							: "Run code and get instant help",
				},
				{
					icon: Award,
					text:
						lang === "ko"
							? "워크숍 완료 시 자동 수료증 발급"
							: "Auto-generated certificates upon completion",
				},
				{
					icon: CheckCircle2,
					text:
						lang === "ko"
							? "개인별 학습 진행 상태 자동 저장"
							: "Automatic progress saving per user",
				},
			],
		},
	});

	function toggleLang() {
		lang = lang === "ko" ? "en" : "ko";
	}
</script>

<svelte:head>
	<title
		>Open Codelabs - {lang === "ko"
			? "인터랙티브 코딩 워크숍 플랫폼"
			: "Interactive Hands-on Workshop Platform"}</title
	>
	<meta
		name="description"
		content="Host, create, and participate in interactive coding workshops with AI-assisted content generation."
	/>
</svelte:head>

<div
	class="min-h-screen bg-neutral-50 text-neutral-900 font-sans selection:bg-blue-100"
>
	<!-- Navigation -->
	<nav
		class="sticky top-0 z-50 bg-white/80 backdrop-blur-md border-b border-neutral-200"
	>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex justify-between h-16 items-center">
				<div class="flex items-center gap-2">
					<div
						class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center"
					>
						<Rocket class="text-white w-5 h-5" />
					</div>
					<span class="text-xl font-bold tracking-tight"
						>Open Codelabs</span
					>
				</div>
				<div
					class="hidden md:flex items-center gap-6 text-sm font-medium text-neutral-600"
				>
					<a
						href="#features"
						class="hover:text-blue-600 transition-colors"
						>{content.nav.features}</a
					>
					<a
						href="#roles"
						class="hover:text-blue-600 transition-colors"
						>{content.nav.roles}</a
					>
					<a
						href="#quickstart"
						class="hover:text-blue-600 transition-colors"
						>{content.nav.quickstart}</a
					>
					<button
						onclick={toggleLang}
						class="flex items-center gap-1 px-3 py-1 rounded-full border border-neutral-200 hover:bg-neutral-50 transition-colors"
					>
						<Globe class="w-4 h-4" />
						{lang === "ko" ? "English" : "한국어"}
					</button>
					<a
						href="https://jaichangpark.github.io/open-codelabs/"
						class="px-5 py-2 bg-neutral-900 text-white rounded-full hover:bg-neutral-800 transition-all hover:scale-105 active:scale-95"
					>
						{content.nav.getStarted}
					</a>
				</div>
			</div>
		</div>
	</nav>

	<!-- Hero Section -->
	<header class="relative overflow-hidden py-24 sm:py-32">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
			<div class="text-center">
				<div
					class="inline-flex items-center px-3 py-1 rounded-full bg-blue-50 border border-blue-100 text-blue-600 text-sm font-medium mb-6 animate-fade-in"
				>
					<Zap class="w-4 h-4 mr-2" />
					{content.hero.badge}
				</div>
				<h1
					class="text-5xl sm:text-7xl font-extrabold tracking-tight text-neutral-900 mb-8 leading-[1.1]"
				>
					{content.hero.title1} <br />
					<span
						class="text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-indigo-600 italic font-black"
						>{content.hero.title2}</span
					>
				</h1>
				<p
					class="max-w-2xl mx-auto text-xl text-neutral-600 mb-10 leading-relaxed"
				>
					{content.hero.desc}
				</p>
				<div class="flex flex-col sm:flex-row justify-center gap-4">
					<a
						href="#quickstart"
						class="px-8 py-4 bg-blue-600 text-white rounded-2xl font-bold text-lg hover:bg-blue-700 shadow-xl shadow-blue-500/20 transition-all hover:scale-105 active:scale-95 flex items-center justify-center gap-2"
					>
						{content.hero.ctaPrimary}
						<ChevronRight class="w-5 h-5" />
					</a>
					<a
						href="https://jaichangpark.github.io/open-codelabs/"
						class="px-8 py-4 bg-white border border-neutral-200 text-neutral-900 rounded-2xl font-bold text-lg hover:bg-neutral-50 transition-all flex items-center justify-center gap-2"
					>
						<BookOpen class="w-5 h-5" />
						{content.hero.ctaSecondary}
					</a>
				</div>
			</div>
		</div>

		<!-- Background Decoration -->
		<div
			class="absolute top-0 left-1/2 -translate-x-1/2 w-full h-full -z-0 pointer-events-none opacity-40"
		>
			<div
				class="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-blue-200 blur-[120px] rounded-full"
			></div>
			<div
				class="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-indigo-200 blur-[120px] rounded-full"
			></div>
		</div>
	</header>
	<!-- Features Grid -->
	<section id="features" class="py-24 bg-white">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="text-center mb-16 reveal">
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.features.title}
				</h2>
				<p class="text-neutral-600 max-w-2xl mx-auto">
					{content.features.desc}
				</p>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
				{#each content.features.items as feature}
					<div
						class="p-8 rounded-3xl border border-neutral-100 bg-neutral-50 hover:border-blue-200 hover:bg-white transition-all group hover:shadow-2xl hover:shadow-blue-500/5 reveal"
					>
						<div
							class="w-12 h-12 bg-white rounded-2xl flex items-center justify-center border border-neutral-200 mb-6 group-hover:scale-110 group-hover:bg-blue-600 group-hover:border-blue-600 transition-all duration-300"
						>
							<feature.icon
								class="w-6 h-6 text-blue-600 group-hover:text-white transition-colors"
							/>
						</div>
						<h3 class="text-xl font-bold mb-3">{feature.title}</h3>
						<p class="text-neutral-600 text-sm leading-relaxed">
							{feature.description}
						</p>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<!-- Detailed Roles Section -->
	<section id="roles" class="py-24 bg-neutral-50">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="grid lg:grid-cols-2 gap-12">
				<!-- Facilitator Box -->
				<div
					class="bg-white p-10 rounded-[2.5rem] shadow-sm border border-neutral-100 hover:shadow-xl transition-shadow reveal"
				>
					<div
						class="w-16 h-16 bg-blue-50 text-blue-600 rounded-3xl flex items-center justify-center mb-8"
					>
						<Users class="w-8 h-8" />
					</div>
					<h3 class="text-3xl font-black mb-6 tracking-tight">
						{content.facilitator.title}
					</h3>
					<ul class="space-y-6">
						{#each content.facilitator.features as f}
							<li class="flex items-start gap-4">
								<div
									class="mt-1 w-6 h-6 rounded-full bg-blue-600/10 flex items-center justify-center shrink-0"
								>
									<f.icon class="w-3.5 h-3.5 text-blue-600" />
								</div>
								<p class="text-neutral-700 font-medium">
									{f.text}
								</p>
							</li>
						{/each}
					</ul>
				</div>

				<!-- Attendee Box -->
				<div
					class="bg-white p-10 rounded-[2.5rem] shadow-sm border border-neutral-100 hover:shadow-xl transition-shadow reveal"
					style="transition-delay: 200ms;"
				>
					<div
						class="w-16 h-16 bg-indigo-50 text-indigo-600 rounded-3xl flex items-center justify-center mb-8"
					>
						<BookOpen class="w-8 h-8" />
					</div>
					<h3 class="text-3xl font-black mb-6 tracking-tight">
						{content.attendee.title}
					</h3>
					<ul class="space-y-6">
						{#each content.attendee.features as f}
							<li class="flex items-start gap-4">
								<div
									class="mt-1 w-6 h-6 rounded-full bg-indigo-600/10 flex items-center justify-center shrink-0"
								>
									<f.icon
										class="w-3.5 h-3.5 text-indigo-600"
									/>
								</div>
								<p class="text-neutral-700 font-medium">
									{f.text}
								</p>
							</li>
						{/each}
					</ul>
				</div>
			</div>
		</div>
	</section>

	<!-- Quickstart Section -->
	<section id="quickstart" class="py-24 bg-white overflow-hidden">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="text-center mb-12 reveal">
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.quickstart.title}
				</h2>
				<p class="text-neutral-600 max-w-2xl mx-auto mb-8">
					{content.quickstart.desc}
				</p>

				<!-- Tabs -->
				<div class="inline-flex p-1 bg-neutral-100 rounded-xl mb-8">
					<button
						onclick={() => (quickstartType = "docker")}
						class="px-6 py-2 rounded-lg text-sm font-bold transition-all {quickstartType ===
						'docker'
							? 'bg-white text-blue-600 shadow-sm'
							: 'text-neutral-500 hover:text-neutral-900'}"
					>
						{content.quickstart.tabs.docker}
					</button>
					<button
						onclick={() => (quickstartType = "podman")}
						class="px-6 py-2 rounded-lg text-sm font-bold transition-all {quickstartType ===
						'podman'
							? 'bg-white text-blue-600 shadow-sm'
							: 'text-neutral-500 hover:text-neutral-900'}"
					>
						{content.quickstart.tabs.podman}
					</button>
				</div>
			</div>

			<div class="relative max-w-4xl mx-auto">
				<div class="space-y-8 relative z-10">
					{#each content.quickstart.steps as step, i}
						<div
							class="flex flex-col md:flex-row gap-6 md:gap-12 items-start group reveal"
						>
							<div class="flex flex-col items-center shrink-0">
								<div
									class="w-10 h-10 rounded-full bg-blue-600 text-white flex items-center justify-center font-bold text-lg shadow-lg shadow-blue-500/30 group-hover:scale-110 transition-transform"
								>
									{i + 1}
								</div>
								{#if i < content.quickstart.steps.length - 1}
									<div
										class="w-0.5 h-full bg-neutral-100 mt-4 min-h-[4rem]"
									></div>
								{/if}
							</div>
							<div class="flex-1 pt-1 w-full">
								<h3 class="text-xl font-bold mb-4">
									{step.title}
								</h3>
								{#if step.code}
									<div
										class="bg-neutral-900 rounded-2xl p-6 font-mono text-sm text-neutral-300 shadow-xl border border-neutral-800 relative overflow-hidden group/code"
									>
										<div class="flex gap-1.5 mb-3">
											<div
												class="w-2.5 h-2.5 rounded-full bg-neutral-700"
											></div>
											<div
												class="w-2.5 h-2.5 rounded-full bg-neutral-700"
											></div>
											<div
												class="w-2.5 h-2.5 rounded-full bg-neutral-700"
											></div>
										</div>
										<pre class="overflow-x-auto"><code
												><span class="text-blue-400"
													>$</span
												> {step.code}</code
											></pre>
									</div>
								{:else}
									<div
										class="p-6 rounded-2xl bg-blue-50 border border-blue-100 text-blue-900 font-medium"
									>
										{step.desc}
									</div>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</section>

	<!-- Tech Stack -->
	<section class="py-24 bg-neutral-900 text-white overflow-hidden relative">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
			<div class="grid lg:grid-cols-2 gap-16 items-center">
				<div class="reveal">
					<h2
						class="text-4xl font-black mb-6 italic tracking-tighter"
					>
						Fast. Modern. Reliable.
					</h2>
					<p class="text-neutral-400 text-lg mb-8 leading-relaxed">
						{lang === "ko"
							? "안전하고 강력한 성능을 위해 백엔드는 Rust로 작성되었으며, 프론트엔드는 SvelteKit을 사용하여 직관적이고 반응성 높은 사용자 경험을 제공합니다."
							: "The backend is written in Rust for safety and performance, while the frontend leverages SvelteKit for a highly reactive and intuitive user experience."}
					</p>
					<div class="grid grid-cols-2 gap-4">
						<div
							class="p-4 rounded-2xl bg-white/5 border border-white/10"
						>
							<div
								class="text-blue-400 font-bold mb-1 text-2xl tracking-tighter"
							>
								Rust
							</div>
							<div class="text-xs text-neutral-500">
								Axum & SQLite
							</div>
						</div>
						<div
							class="p-4 rounded-2xl bg-white/5 border border-white/10"
						>
							<div
								class="text-orange-500 font-bold mb-1 text-2xl tracking-tighter"
							>
								Svelte
							</div>
							<div class="text-xs text-neutral-500">
								Runes & Tailwind
							</div>
						</div>
					</div>
				</div>
				<div class="relative reveal" style="transition-delay: 300ms;">
					<div
						class="bg-neutral-800 p-6 rounded-3xl border border-neutral-700 font-mono text-sm overflow-x-auto shadow-2xl scale-105"
					>
						<div class="flex gap-2 mb-4">
							<div
								class="w-3 h-3 rounded-full bg-red-500/50"
							></div>
							<div
								class="w-3 h-3 rounded-full bg-yellow-500/50"
							></div>
							<div
								class="w-3 h-3 rounded-full bg-green-500/50"
							></div>
						</div>
						<pre class="text-neutral-300"><code
								><span class="text-neutral-500">// lib.rs</span>
<span class="text-blue-400">pub async fn</span> <span class="text-yellow-400"
									>start_server</span
								>() &#123;
    <span class="text-neutral-500">// Blazing fast backend</span>
    <span class="text-blue-400">let</span> app = <span class="text-indigo-400"
									>Router</span
								>::<span class="text-yellow-400">new</span>()
        .<span class="text-yellow-400">route</span>(<span class="text-green-400"
									>"/api/codelabs"</span
								>, <span class="text-yellow-400">get</span
								>(list_codelabs))
        .<span class="text-yellow-400">layer</span>(<span
									class="text-indigo-400">CorsLayer</span
								>::<span class="text-yellow-400"
									>permissive</span
								>());

    <span class="text-indigo-400">axum</span>::<span class="text-indigo-400"
									>Server</span
								>::<span class="text-yellow-400">bind</span
								>(<span class="text-green-400"
									>&amp;"0.0.0.0:8080"</span
								>.<span class="text-yellow-400">parse</span
								>().<span class="text-yellow-400">unwrap</span
								>())
        .<span class="text-yellow-400">serve</span>(app.<span
									class="text-yellow-400"
									>into_make_service</span
								>())
        .<span class="text-blue-400">await</span>
        .<span class="text-yellow-400">unwrap</span>();
&#125;</code
							></pre>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Footer -->
	<footer class="bg-white py-16 border-t border-neutral-100">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div
				class="flex flex-col md:flex-row justify-between items-center gap-8"
			>
				<div class="flex items-center gap-3">
					<div
						class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center"
					>
						<Rocket class="text-white w-6 h-6" />
					</div>
					<span class="text-xl font-black tracking-tight"
						>Open Codelabs</span
					>
				</div>
				<p class="text-neutral-500 text-sm font-medium">
					© 2025 Open Codelabs Project. Released under MIT License.
				</p>
				<div class="flex gap-6">
					<a
						href="https://github.com/jaichangpark/open-codelabs"
						class="text-neutral-400 hover:text-blue-600 transition-all hover:scale-110"
					>
						<Github class="w-6 h-6" />
					</a>
				</div>
			</div>
		</div>
	</footer>
</div>

<style>
	:global(html) {
		scroll-behavior: smooth;
	}

	@keyframes fade-in {
		from {
			opacity: 0;
			transform: translateY(10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.animate-fade-in {
		animation: fade-in 0.8s ease-out forwards;
	}
</style>
