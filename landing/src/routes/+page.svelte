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
		ClipboardCheck,
		FileText,
		FolderTree,
		GitBranch,
		LifeBuoy,
		QrCode,
		Sparkles,
		Sun,
		Moon,
	} from "lucide-svelte";
	import LetterGlitch from "$lib/components/LetterGlitch.svelte";

	let lang = $state("ko");
	let isDark = $state(true); // Default to dark mode
	let quickstartType = $state("docker"); // 'docker' or 'podman'

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				entries.forEach((entry) => {
					if (entry.isIntersecting) {
						(entry.target as HTMLElement).dataset.revealed = "true";
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
			liveOps: lang === "ko" ? "라이브 운영" : "Live Ops",
			quickstart: lang === "ko" ? "빠른 시작" : "Quickstart",
			getStarted: lang === "ko" ? "시작하기" : "Get Started",
		},
		hero: {
			badge: lang === "ko" ? "AI 기반" : "Powered by Gemini",
			title1: lang === "ko" ? "더 스마트한" : "Interactive Workshops",
			title2: lang === "ko" ? "코딩 워크숍의 시작." : "Made Simple.",
			desc:
				lang === "ko"
					? "Open Codelabs는 현대적인 오픈소스 코딩 워크숍 플랫폼입니다.\nGoogle Codelabs의 경험을 AI와 함께 더욱 강력하게 만들었습니다."
					: "Open Codelabs is a modern, open-source platform for hosting hands-on coding workshops. Inspired by Google Codelabs, enhanced with AI.",
			ctaPrimary:
				lang === "ko" ? "5분 만에 시작하기" : "Get Started in 5m",
			ctaSecondary: lang === "ko" ? "문서 보기" : "View Docs",
		},
		valueProps: {
			title:
				lang === "ko" ? "왜 Open Codelabs인가?" : "Why Open Codelabs",
			desc:
				lang === "ko"
					? "AI 작성, 실시간 운영, 보안을 한 번에 갖춘 실습 플랫폼입니다."
					: "AI-native authoring, live operations, and security in one platform.",
			items: [
				{
					icon: Sparkles,
					title:
						lang === "ko" ? "AI-우선 작성" : "AI-first authoring",
					description:
						lang === "ko"
							? "Gemini 기반 코드랩 생성·요약·리뷰로 워크숍 콘텐츠를 빠르게 반복합니다."
							: "Gemini-powered generation, summaries, and reviews accelerate workshop content.",
				},
				{
					icon: BarChart3,
					title:
						lang === "ko"
							? "운영 콘트롤 타워"
							: "Operational control tower",
					description:
						lang === "ko"
							? "실시간 참석자 상태, 도움 요청, 제출물, 퀴즈를 한 화면에서 모니터링합니다."
							: "Monitor attendee status, help requests, submissions, and quizzes in one view.",
				},
				{
					icon: Shield,
					title:
						lang === "ko"
							? "보안과 데이터 주권"
							: "Security & data ownership",
					description:
						lang === "ko"
							? "암호화된 API 키, 프라이빗 워크숍, 자체 호스팅 옵션으로 데이터 통제권을 유지하세요."
							: "Encrypted keys, private workshops, and self-hosting keep you in control.",
				},
			],
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
		liveOps: {
			eyebrow: lang === "ko" ? "라이브 운영 키트" : "Live Ops Toolkit",
			title:
				lang === "ko"
					? "세션 현장에서 바로 쓰는 운영 도구"
					: "Run the room with real-time tools",
			desc:
				lang === "ko"
					? "채팅, 도움 요청, 퀴즈·피드백, 자료 배포, QR 초대를 한 화면에서 운영합니다."
					: "Chat, help queues, quizzes, materials, and QR invites in one view.",
			items: [
				{
					icon: MessageSquare,
					title: lang === "ko" ? "실시간 채팅 & 1:1 DM" : "Live chat & 1:1 DM",
					description:
						lang === "ko"
							? "공개 채팅과 개인 메시지로 질문을 즉시 해결합니다."
							: "Answer questions instantly with public chat and direct messages.",
					badges:
						lang === "ko"
							? ["공개 채팅", "1:1 DM"]
							: ["Public chat", "Direct messages"],
				},
				{
					icon: LifeBuoy,
					title:
						lang === "ko"
							? "도움 요청 큐 & 제출물"
							: "Help queue & submissions",
					description:
						lang === "ko"
							? "막힌 참가자 요청을 큐로 모아 제출물까지 한 번에 확인합니다."
							: "Collect stuck participants and review submissions in one place.",
					badges:
						lang === "ko"
							? ["도움 요청", "제출물 패널"]
							: ["Help queue", "Submission panel"],
				},
				{
					icon: ClipboardCheck,
					title:
						lang === "ko"
							? "퀴즈·피드백·수료증"
							: "Quizzes, feedback, certificates",
					description:
						lang === "ko"
							? "퀴즈/피드백 완료를 수료 조건으로 설정할 수 있습니다."
							: "Gate completion on quizzes and feedback, then auto-issue certificates.",
					badges:
						lang === "ko"
							? ["퀴즈", "피드백", "수료증"]
							: ["Quiz", "Feedback", "Certificate"],
				},
				{
					icon: FileText,
					title:
						lang === "ko"
							? "준비 가이드 & 자료 배포"
							: "Prep guides & materials",
					description:
						lang === "ko"
							? "세션 전 준비 안내와 링크/파일을 한 곳에서 배포합니다."
							: "Share prework guidance plus links and files in one hub.",
					badges:
						lang === "ko"
							? ["가이드", "링크/파일"]
							: ["Guides", "Links/files"],
				},
				{
					icon: QrCode,
					title:
						lang === "ko"
							? "QR 초대 & 공개 공유"
							: "QR invites & public links",
					description:
						lang === "ko"
							? "ngrok/bore/cloudflared로 공개하고 QR로 빠르게 초대합니다."
							: "Expose with ngrok, bore, or cloudflared and invite via QR.",
					badges:
						lang === "ko"
							? ["QR 코드", "터널링"]
							: ["QR code", "Tunneling"],
				},
			],
		},
		aiModes: {
			eyebrow: lang === "ko" ? "AI 생성 모드" : "AI Generation Modes",
			title:
				lang === "ko"
					? "필요한 깊이만큼 코드랩을 설계하세요"
					: "Generate codelabs with the depth you need",
			desc:
				lang === "ko"
					? "일반 모드로 빠르게 초안을 만들거나, 프로 모드로 플랜과 리뷰까지 포함한 완성도를 확보하세요."
					: "Use General mode for speed or Pro mode for planning, review, and refinement.",
			modes: [
				{
					icon: Zap,
					title: lang === "ko" ? "일반 모드" : "General mode",
					desc:
						lang === "ko"
							? "코드와 첨부파일을 기준으로 단일 패스에서 초안을 생성합니다."
							: "One-pass generation from code and attachments for a fast draft.",
					badges:
						lang === "ko"
							? ["단일 패스", "빠른 생성", "즉시 초안"]
							: ["Single pass", "Fast", "Instant draft"],
				},
				{
					icon: Sparkles,
					title: lang === "ko" ? "프로 모드" : "Pro mode",
					desc:
						lang === "ko"
							? "플랜 설계, 전문가 리뷰, 수정 반영까지 포함한 고품질 워크플로."
							: "Plan, expert review, and revision for a high-fidelity output.",
					badges:
						lang === "ko"
							? ["플랜", "전문가 리뷰", "수정 반영"]
							: ["Plan", "Expert review", "Revise"],
				},
			],
			workflow: {
					title:
						lang === "ko" ? "프로 생성 워크플로" : "Pro workflow",
				desc:
					lang === "ko"
						? "플랜 단계에서 최신 정보 키워드를 추출하고, 생성 단계에서 검색을 활용해 신뢰도를 높입니다."
						: "Extract search terms during planning and enrich drafts with live lookup.",
				steps: [
					{
						title: lang === "ko" ? "플랜" : "Plan",
						desc:
							lang === "ko"
								? "목표, 환경, 검증 포인트와 검색어 정의"
								: "Define goals, setup, verification, and search terms",
					},
					{
						title: lang === "ko" ? "초안" : "Draft",
						desc:
							lang === "ko"
								? "플랜 기반으로 코드랩 구조를 생성"
								: "Generate the codelab structure from the plan",
					},
					{
						title: lang === "ko" ? "리뷰" : "Review",
						desc:
							lang === "ko"
								? "퍼실리테이터 관점으로 품질 점검"
								: "Expert facilitator review for gaps and clarity",
					},
					{
						title: lang === "ko" ? "수정" : "Revise",
						desc:
							lang === "ko"
								? "리뷰 반영 후 최종 결과 확정"
								: "Apply fixes and finalize the output",
					},
				],
			},
		},
		proReady: {
			eyebrow: lang === "ko" ? "프로답게" : "Pro-Grade",
			title:
				lang === "ko"
					? "사전 준비까지 AI로 완성"
					: "Prep like a pro with AI",
			desc:
				lang === "ko"
					? "세션 전에 필요한 환경, 플러그인, 자료를 AI가 정리하고 공유 가능한 준비 가이드로 묶어줍니다."
					: "Let AI compile environment setup, plugins, and materials into a polished prep guide.",
			flow: {
				title:
					lang === "ko"
						? "사전 준비 AI 워크플로"
						: "AI preflight workflow",
				desc:
					lang === "ko"
						? "플랜부터 가이드 생성, 검토, 배포까지 한 번에 진행합니다."
						: "Go from plan to guide, review, and share in one flow.",
				steps: [
					{
						icon: Sparkles,
						title: lang === "ko" ? "플랜" : "Plan",
						desc:
							lang === "ko"
								? "목표, 난이도, 환경 요구사항을 정리"
								: "Define goals, level, and environment needs",
					},
					{
						icon: FileText,
						title: lang === "ko" ? "가이드 초안" : "Draft guide",
						desc:
							lang === "ko"
								? "설치/설정/IDE 안내를 자동 생성"
								: "Generate install, config, and IDE setup",
					},
					{
						icon: ClipboardCheck,
						title: lang === "ko" ? "검토·보완" : "Review & refine",
						desc:
							lang === "ko"
								? "누락과 충돌 포인트를 점검"
								: "Patch gaps and remove friction",
					},
					{
						icon: Rocket,
						title: lang === "ko" ? "배포" : "Share",
						desc:
							lang === "ko"
								? "링크와 파일을 준비 자료로 공유"
								: "Distribute links and files to attendees",
					},
				],
				badges:
					lang === "ko"
						? ["준비 가이드", "환경 체크", "공지 템플릿"]
						: ["Prep guide", "Environment checks", "Share-ready"],
			},
			workspace: {
				eyebrow: lang === "ko" ? "AI 워크스페이스" : "AI workspace",
				title:
					lang === "ko"
						? "워크스페이스까지 한 번에"
						: "Workspace built alongside the codelab",
				desc:
					lang === "ko"
						? "AI 생성 흐름에서 업로드한 파일을 기준으로 Code Server 워크스페이스 구조를 자동 구성합니다."
						: "During AI generation, build a Code Server workspace from your uploaded files.",
				branch: {
					title: lang === "ko" ? "브랜치 방식" : "Branch mode",
					desc:
						lang === "ko"
							? "단계별 start/end 브랜치로 상태 전환을 명확하게."
							: "Start/end branches per step for clean diffs.",
					nodes: [
						"main",
						"step-1/start",
						"step-1/end",
						"step-2/start",
						"step-2/end",
					],
				},
				folder: {
					title: lang === "ko" ? "폴더 방식" : "Folder mode",
					desc:
						lang === "ko"
							? "디렉토리 트리로 시작/완료 버전을 직관적으로."
							: "Structured folders for start/end snapshots.",
					tree: [
						{ depth: 0, label: "workspace/" },
						{ depth: 1, label: "step-1/" },
						{ depth: 2, label: "start/" },
						{ depth: 2, label: "end/" },
						{ depth: 1, label: "step-2/" },
						{ depth: 2, label: "start/" },
						{ depth: 2, label: "end/" },
					],
				},
				badges:
					lang === "ko"
						? ["Code Server", "브랜치/폴더", "Step start/end"]
						: ["Code Server", "Branch/Folder", "Step start/end"],
			},
		},
		deployment: {
			title: lang === "ko" ? "원하는 방식으로 배포" : "Deploy your way",
			desc:
				lang === "ko"
					? "로컬 Docker, 클라우드, 또는 Firebase 서버리스까지 팀 환경에 맞춰 선택하세요."
					: "Run locally with Docker, in your cloud, or serverless with Firebase—pick what fits your team.",
			options: [
				{
					title: lang === "ko" ? "자체 호스팅" : "Self-hosted",
					description:
						lang === "ko"
							? "Rust + SQLite 조합으로 가볍고 빠른 단일 바이너리. 사내 네트워크에 띄워도 됩니다."
							: "Rust + SQLite, lightweight single binary. Perfect for on-prem or locked-down networks.",
					badges: ["Docker", "Podman", "Axum"],
				},
				{
					title: "Firebase",
					description:
						lang === "ko"
							? "서버리스 배포로 인증, DB, 스토리지를 간단히 운영. 최소한의 인프라 관리."
							: "Serverless deployment with auth, DB, and storage—minimal infra to manage.",
					badges: ["Hosting", "Firestore", "Storage"],
				},
			],
		},
	});

	function toggleLang() {
		lang = lang === "ko" ? "en" : "ko";
	}

	function toggleTheme() {
		isDark = !isDark;
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
	class="min-h-screen transition-colors duration-300 {isDark
		? 'bg-neutral-950 text-white'
		: 'bg-neutral-50 text-neutral-900'} font-sans selection:bg-blue-500/30"
>
	<!-- Navigation -->
	<nav
		class="sticky top-0 z-50 border-b transition-colors duration-300 {isDark
			? 'bg-neutral-950/80 border-neutral-800'
			: 'bg-white/80 border-neutral-200'} backdrop-blur-md"
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
					class="hidden md:flex items-center gap-6 text-sm font-medium {isDark
						? 'text-neutral-400'
						: 'text-neutral-600'}"
				>
					<a
						href="#features"
						class="hover:text-blue-500 transition-colors"
						>{content.nav.features}</a
					>
					<a
						href="#roles"
						class="hover:text-blue-500 transition-colors"
						>{content.nav.roles}</a
					>
					<a
						href="#live-ops"
						class="hover:text-blue-500 transition-colors"
						>{content.nav.liveOps}</a
					>
					<a
						href="#quickstart"
						class="hover:text-blue-500 transition-colors"
						>{content.nav.quickstart}</a
					>
					<button
						onclick={toggleLang}
						class="flex items-center gap-1 px-3 py-1 rounded-full border transition-colors {isDark
							? 'border-neutral-800 hover:bg-neutral-900'
							: 'border-neutral-200 hover:bg-neutral-50'}"
					>
						<Globe class="w-4 h-4" />
						{lang === "ko" ? "English" : "한국어"}
					</button>
					<button
						onclick={toggleTheme}
						class="p-2 rounded-full border transition-colors {isDark
							? 'border-neutral-800 hover:bg-neutral-900 text-yellow-400'
							: 'border-neutral-200 hover:bg-neutral-50 text-blue-600'}"
						aria-label="Toggle Theme"
					>
						{#if isDark}
							<Sun class="w-4 h-4" />
						{:else}
							<Moon class="w-4 h-4" />
						{/if}
					</button>
					<a
						href="https://jaichangpark.github.io/open-codelabs/"
						class="px-5 py-2 rounded-full transition-all hover:scale-105 active:scale-95 {isDark
							? 'bg-white text-neutral-950 hover:bg-neutral-200'
							: 'bg-neutral-900 text-white hover:bg-neutral-800'}"
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
					class="text-5xl sm:text-7xl font-extrabold tracking-tight mb-8 leading-[1.1] {isDark
						? 'text-white'
						: 'text-neutral-900'}"
				>
					{content.hero.title1} <br />
					<span
						class="text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-indigo-400 italic font-black"
						>{content.hero.title2}</span
					>
				</h1>
				<p
					class="max-w-2xl mx-auto text-xl mb-10 leading-relaxed whitespace-pre-line relative z-10 {isDark
						? 'text-neutral-400'
						: 'text-neutral-800 font-medium'}"
				>
					{content.hero.desc}
				</p>
				<div
					class="flex flex-col sm:flex-row justify-center gap-4 relative z-10"
				>
					<a
						href="#quickstart"
						class="px-8 py-4 bg-blue-600 text-white rounded-2xl font-bold text-lg hover:bg-blue-700 shadow-xl shadow-blue-500/20 transition-all hover:scale-105 active:scale-95 flex items-center justify-center gap-2"
					>
						{content.hero.ctaPrimary}
						<ChevronRight class="w-5 h-5" />
					</a>
					<a
						href="https://jaichangpark.github.io/open-codelabs/"
						class="px-8 py-4 border rounded-2xl font-bold text-lg transition-all flex items-center justify-center gap-2 {isDark
							? 'bg-neutral-900/50 border-neutral-700 text-white hover:bg-neutral-800'
							: 'bg-white/80 border-neutral-200 text-neutral-900 hover:bg-neutral-50 backdrop-blur-sm'}"
					>
						<BookOpen class="w-5 h-5" />
						{content.hero.ctaSecondary}
					</a>
				</div>
			</div>
		</div>

		<!-- Background Glitch Decoration -->
		<div
			class="absolute inset-0 -z-0 pointer-events-none transition-opacity duration-500 {isDark
				? 'opacity-50'
				: 'opacity-20'}"
		>
			<LetterGlitch
				glitchColors={isDark
					? ["#1e293b", "#3b82f6", "#4f46e5"]
					: ["#f0f9ff", "#e0f2fe", "#bae6fd"]}
				outerVignette={true}
				centerVignette={true}
				smooth={true}
				{isDark}
			/>
		</div>
	</header>

	<section
		class="py-20 transition-colors duration-300 {isDark
			? 'bg-neutral-900'
			: 'bg-neutral-50'}"
	>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="text-center mb-12 reveal">
				<p
					class="text-blue-500 font-bold uppercase tracking-[0.2em] text-xs mb-3"
				>
					{lang === "ko" ? "핵심 가치" : "VALUE PILLARS"}
				</p>
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.valueProps.title}
				</h2>
				<p
					class="{isDark
						? 'text-neutral-400'
						: 'text-neutral-600'} max-w-2xl mx-auto"
				>
					{content.valueProps.desc}
				</p>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
				{#each content.valueProps.items as item, i}
					<div
						class="p-8 rounded-3xl border transition-all reveal {isDark
							? 'bg-neutral-800/50 border-neutral-700 shadow-xl hover:bg-neutral-800'
							: 'bg-white border-neutral-100 shadow-sm hover:shadow-xl'}"
						style={`transition-delay:${i * 100}ms`}
					>
						<div
							class="w-12 h-12 rounded-2xl flex items-center justify-center mb-5 {isDark
								? 'bg-blue-500/10 text-blue-400'
								: 'bg-blue-50 text-blue-600'}"
						>
							<item.icon class="w-6 h-6" />
						</div>
						<h3 class="text-xl font-bold mb-3">{item.title}</h3>
						<p
							class="leading-relaxed text-sm {isDark
								? 'text-neutral-400'
								: 'text-neutral-600'}"
						>
							{item.description}
						</p>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<section
		id="features"
		class="py-24 transition-colors duration-300 {isDark
			? 'bg-neutral-950'
			: 'bg-white'}"
	>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="text-center mb-16 reveal">
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.features.title}
				</h2>
				<p
					class="{isDark
						? 'text-neutral-400'
						: 'text-neutral-600'} max-w-2xl mx-auto"
				>
					{content.features.desc}
				</p>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
				{#each content.features.items as feature}
					<div
						class="p-8 rounded-3xl border transition-all group hover:shadow-2xl reveal {isDark
							? 'border-neutral-800 bg-neutral-900/50 hover:border-blue-500/50 hover:bg-neutral-800 hover:shadow-blue-500/10'
							: 'border-neutral-100 bg-neutral-50 hover:border-blue-200 hover:bg-white hover:shadow-blue-500/5'}"
					>
						<div
							class="w-12 h-12 rounded-2xl flex items-center justify-center border transition-all duration-300 {isDark
								? 'bg-neutral-800 border-neutral-700 group-hover:bg-blue-500 group-hover:border-blue-500'
								: 'bg-white border-neutral-200 group-hover:bg-blue-600 group-hover:border-blue-600'}"
						>
							<feature.icon
								class="w-6 h-6 transition-colors {isDark
									? 'text-blue-400 group-hover:text-white'
									: 'text-blue-600 group-hover:text-white'}"
							/>
						</div>
						<h3 class="text-xl font-bold mb-3">{feature.title}</h3>
						<p
							class="text-sm leading-relaxed {isDark
								? 'text-neutral-400'
								: 'text-neutral-600'}"
						>
							{feature.description}
						</p>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<section
		id="roles"
		class="py-24 transition-colors duration-300 {isDark
			? 'bg-neutral-900'
			: 'bg-neutral-50'}"
	>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="grid lg:grid-cols-2 gap-12">
				<!-- Facilitator Box -->
				<div
					class="p-10 rounded-[2.5rem] reveal transition-all {isDark
						? 'bg-neutral-800/50 border border-neutral-700 shadow-2xl'
						: 'bg-white border border-neutral-100 shadow-sm'}"
				>
					<div
						class="w-16 h-16 rounded-3xl flex items-center justify-center mb-8 {isDark
							? 'bg-blue-500/10 text-blue-400'
							: 'bg-blue-50 text-blue-600'}"
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
									class="mt-1 w-6 h-6 rounded-full flex items-center justify-center shrink-0 {isDark
										? 'bg-blue-400/10 text-blue-400'
										: 'bg-blue-600/10 text-blue-600'}"
								>
									<f.icon class="w-3.5 h-3.5" />
								</div>
								<p
									class="{isDark
										? 'text-neutral-300'
										: 'text-neutral-700'} font-medium"
								>
									{f.text}
								</p>
							</li>
						{/each}
					</ul>
				</div>

				<!-- Attendee Box -->
				<div
					class="p-10 rounded-[2.5rem] reveal transition-all {isDark
						? 'bg-neutral-800/50 border border-neutral-700 shadow-2xl'
						: 'bg-white border border-neutral-100 shadow-sm'}"
					style="transition-delay: 200ms;"
				>
					<div
						class="w-16 h-16 rounded-3xl flex items-center justify-center mb-8 {isDark
							? 'bg-indigo-500/10 text-indigo-400'
							: 'bg-indigo-50 text-indigo-600'}"
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
									class="mt-1 w-6 h-6 rounded-full flex items-center justify-center shrink-0 {isDark
										? 'bg-indigo-400/10 text-indigo-400'
										: 'bg-indigo-600/10 text-indigo-600'}"
								>
									<f.icon class="w-3.5 h-3.5" />
								</div>
								<p
									class="{isDark
										? 'text-neutral-300'
										: 'text-neutral-700'} font-medium"
								>
									{f.text}
								</p>
							</li>
						{/each}
					</ul>
				</div>
			</div>
		</div>
	</section>

	<section
		id="live-ops"
		class="py-24 transition-colors duration-300 relative overflow-hidden {isDark
			? 'bg-neutral-950'
			: 'bg-white'}"
	>
		<div
			class="pointer-events-none absolute -top-24 right-0 h-72 w-72 translate-x-1/3 rounded-full blur-[120px] opacity-50 {isDark
				? 'bg-blue-500/20'
				: 'bg-blue-200/70'}"
		></div>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
			<div class="text-center mb-14 reveal">
				<p
					class="text-blue-500 font-bold uppercase tracking-[0.2em] text-xs mb-3"
				>
					{content.liveOps.eyebrow}
				</p>
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.liveOps.title}
				</h2>
				<p
					class="{isDark
						? 'text-neutral-400'
						: 'text-neutral-600'} max-w-2xl mx-auto"
				>
					{content.liveOps.desc}
				</p>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
				{#each content.liveOps.items as item, i}
					<div
						class="p-7 rounded-3xl border transition-all reveal {isDark
							? 'bg-neutral-900/60 border-neutral-800 hover:border-blue-500/40'
							: 'bg-neutral-50 border-neutral-200 hover:border-blue-300 hover:bg-white shadow-sm hover:shadow-xl'}"
						style={`transition-delay:${i * 90}ms`}
					>
						<div
							class="w-12 h-12 rounded-2xl flex items-center justify-center mb-5 {isDark
								? 'bg-blue-500/10 text-blue-400'
								: 'bg-blue-50 text-blue-600'}"
						>
							<item.icon class="w-6 h-6" />
						</div>
						<h3 class="text-xl font-bold mb-3">{item.title}</h3>
						<p
							class="text-sm leading-relaxed {isDark
								? 'text-neutral-400'
								: 'text-neutral-600'}"
						>
							{item.description}
						</p>
						<div class="mt-4 flex flex-wrap gap-2">
							{#each item.badges as badge}
								<span
									class="px-3 py-1 rounded-full border text-[11px] font-bold uppercase tracking-wide {isDark
										? 'bg-neutral-950 border-neutral-800 text-neutral-400'
										: 'bg-white border-neutral-200 text-neutral-600'}"
								>
									{badge}
								</span>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<section
		id="ai-modes"
		class="py-24 transition-colors duration-300 relative overflow-hidden {isDark
			? 'bg-neutral-950'
			: 'bg-white'}"
	>
		<div
			class="pointer-events-none absolute -top-32 left-1/2 h-80 w-80 -translate-x-1/2 rounded-full blur-[120px] opacity-60 {isDark
				? 'bg-blue-500/20'
				: 'bg-blue-200/70'}"
		></div>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
			<div class="grid lg:grid-cols-[1.05fr_0.95fr] gap-12 items-center">
				<div class="space-y-6 reveal">
					<p
						class="text-blue-500 font-bold uppercase tracking-[0.2em] text-xs"
					>
						{content.aiModes.eyebrow}
					</p>
					<h2 class="text-3xl sm:text-4xl font-black tracking-tight">
						{content.aiModes.title}
					</h2>
					<p
						class="leading-relaxed {isDark
							? 'text-neutral-400'
							: 'text-neutral-600'}"
					>
						{content.aiModes.desc}
					</p>
					<div class="grid sm:grid-cols-2 gap-4">
						{#each content.aiModes.modes as mode, i}
							<div
								class="p-6 rounded-3xl border transition-all reveal {isDark
									? 'bg-neutral-900/60 border-neutral-800 hover:border-blue-500/40'
									: 'bg-neutral-50 border-neutral-200 hover:border-blue-300 hover:bg-white'}"
								style={`transition-delay:${i * 120}ms`}
							>
								<div class="flex items-center gap-3 mb-4">
									<div
										class="w-10 h-10 rounded-2xl flex items-center justify-center {isDark
											? 'bg-blue-500/10 text-blue-400'
											: 'bg-blue-50 text-blue-600'}"
									>
										<mode.icon class="w-5 h-5" />
									</div>
									<h3 class="text-lg font-black">
										{mode.title}
									</h3>
								</div>
								<p
									class="text-sm leading-relaxed {isDark
										? 'text-neutral-400'
										: 'text-neutral-600'}"
								>
									{mode.desc}
								</p>
								<div class="mt-4 flex flex-wrap gap-2">
									{#each mode.badges as badge}
										<span
											class="px-3 py-1 rounded-full border text-[11px] font-bold uppercase tracking-wide {isDark
												? 'bg-neutral-900 border-neutral-700 text-neutral-400'
												: 'bg-white border-neutral-200 text-neutral-600'}"
										>
											{badge}
										</span>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				</div>

				<div
					class="workflow-card reveal rounded-[2.5rem] border p-8 relative overflow-hidden {isDark
						? 'bg-neutral-900/60 border-neutral-800'
						: 'bg-white border-neutral-200 shadow-sm'}"
				>
					<div class="workflow-sheen"></div>
					<div class="relative z-10 space-y-4">
						<div
							class="flex items-center gap-2 text-xs font-bold uppercase tracking-[0.2em] text-blue-500"
						>
							<Sparkles class="w-4 h-4" />
							{content.aiModes.workflow.title}
						</div>
						<p
							class="text-sm leading-relaxed {isDark
								? 'text-neutral-400'
								: 'text-neutral-600'}"
						>
							{content.aiModes.workflow.desc}
						</p>
					</div>
					<div class="workflow-steps relative mt-6 space-y-4">
						<div class="workflow-rail"></div>
						{#each content.aiModes.workflow.steps as step, i}
							<div
								class="workflow-step rounded-2xl border p-4 transition-colors {isDark
									? 'bg-neutral-950/60 border-neutral-800'
									: 'bg-neutral-50 border-neutral-200'}"
								style={`--delay:${i * 0.4}s; transition-delay:${i * 120}ms`}
							>
								<div class="workflow-dot"></div>
								<div class="space-y-1">
									<p class="text-sm font-bold">{step.title}</p>
									<p
										class="text-xs leading-relaxed {isDark
											? 'text-neutral-400'
											: 'text-neutral-600'}"
									>
										{step.desc}
									</p>
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>
	</section>

	<section
		id="pro-ready"
		class="py-24 transition-colors duration-300 relative overflow-hidden {isDark
			? 'bg-neutral-900'
			: 'bg-neutral-50'}"
	>
		<div
			class="pointer-events-none absolute -top-24 left-0 h-72 w-72 -translate-x-1/3 rounded-full blur-[120px] opacity-60 {isDark
				? 'bg-blue-500/20'
				: 'bg-blue-200/70'}"
		></div>
		<div
			class="pointer-events-none absolute bottom-0 right-0 h-72 w-72 translate-x-1/3 rounded-full blur-[140px] opacity-50 {isDark
				? 'bg-emerald-500/10'
				: 'bg-emerald-200/60'}"
		></div>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
			<div class="grid lg:grid-cols-[1.05fr_0.95fr] gap-12 items-start">
				<div class="space-y-8 reveal">
					<p
						class="text-blue-500 font-bold uppercase tracking-[0.2em] text-xs"
					>
						{content.proReady.eyebrow}
					</p>
					<h2 class="text-3xl sm:text-4xl font-black tracking-tight">
						{content.proReady.title}
					</h2>
					<p
						class="leading-relaxed {isDark
							? 'text-neutral-400'
							: 'text-neutral-600'}"
					>
						{content.proReady.desc}
					</p>

					<div
						class="prep-card rounded-[2.5rem] border p-8 transition-colors {isDark
							? 'bg-neutral-950/70 border-neutral-800'
							: 'bg-white border-neutral-200 shadow-sm'}"
					>
						<div class="flex items-start gap-4">
							<div
								class="w-12 h-12 rounded-2xl flex items-center justify-center {isDark
									? 'bg-blue-500/10 text-blue-300'
									: 'bg-blue-50 text-blue-600'}"
							>
								<Sparkles class="w-6 h-6" />
							</div>
							<div class="space-y-2">
								<p
									class="text-sm font-bold uppercase tracking-[0.2em] text-blue-500"
								>
									{content.proReady.flow.title}
								</p>
								<p
									class="text-sm leading-relaxed {isDark
										? 'text-neutral-400'
										: 'text-neutral-600'}"
								>
									{content.proReady.flow.desc}
								</p>
							</div>
						</div>

						<div class="prep-flow mt-6 space-y-4">
							<div class="prep-rail"></div>
							{#each content.proReady.flow.steps as step, i}
								<div
									class="prep-step"
									style={`--delay:${i * 0.35}s`}
								>
									<span class="prep-dot"></span>
									<div
										class="prep-step-icon rounded-2xl flex items-center justify-center {isDark
											? 'bg-neutral-900 text-blue-300'
											: 'bg-blue-50 text-blue-600'}"
									>
										<step.icon class="w-4 h-4" />
									</div>
									<div class="space-y-1">
										<p class="text-sm font-bold">
											{step.title}
										</p>
										<p
											class="text-xs leading-relaxed {isDark
												? 'text-neutral-400'
												: 'text-neutral-600'}"
										>
											{step.desc}
										</p>
									</div>
								</div>
							{/each}
						</div>

						<div class="mt-6 flex flex-wrap gap-2">
							{#each content.proReady.flow.badges as badge}
								<span
									class="px-3 py-1 rounded-full border text-[11px] font-bold uppercase tracking-wide {isDark
										? 'bg-neutral-950 border-neutral-800 text-neutral-400'
										: 'bg-neutral-50 border-neutral-200 text-neutral-600'}"
								>
									{badge}
								</span>
							{/each}
						</div>
					</div>
				</div>

				<div class="space-y-6 reveal" style="transition-delay: 200ms;">
					<div
						class="workspace-card rounded-[2.5rem] border p-8 relative overflow-hidden {isDark
							? 'bg-neutral-950/70 border-neutral-800'
							: 'bg-white border-neutral-200 shadow-sm'}"
					>
						<div
							class="pointer-events-none absolute -right-20 top-10 h-48 w-48 rounded-full blur-[80px] opacity-60 {isDark
								? 'bg-blue-500/20'
								: 'bg-blue-200/70'}"
						></div>
						<div class="relative z-10">
							<p
								class="text-blue-500 font-bold uppercase tracking-[0.2em] text-xs"
							>
								{content.proReady.workspace.eyebrow}
							</p>
							<div class="mt-4 flex items-start justify-between gap-4">
								<div>
									<h3 class="text-2xl font-black mb-3">
										{content.proReady.workspace.title}
									</h3>
									<p
										class="text-sm leading-relaxed {isDark
											? 'text-neutral-400'
											: 'text-neutral-600'}"
									>
										{content.proReady.workspace.desc}
									</p>
								</div>
								<div
									class="w-12 h-12 rounded-2xl flex items-center justify-center {isDark
										? 'bg-blue-500/10 text-blue-300'
										: 'bg-blue-50 text-blue-600'}"
								>
									<Sparkles class="w-5 h-5" />
								</div>
							</div>

							<div class="grid sm:grid-cols-2 gap-4 mt-6">
								<div
									class="workspace-panel rounded-3xl border p-5 transition-colors {isDark
										? 'bg-neutral-900/60 border-neutral-800'
										: 'bg-neutral-50 border-neutral-200'}"
								>
									<div class="flex items-center gap-3 mb-3">
										<div
											class="w-9 h-9 rounded-2xl flex items-center justify-center {isDark
												? 'bg-blue-500/10 text-blue-300'
												: 'bg-blue-50 text-blue-600'}"
										>
											<GitBranch class="w-4 h-4" />
										</div>
										<div class="space-y-0.5">
											<p
												class="text-xs font-bold uppercase tracking-[0.2em] text-blue-500"
											>
												{content.proReady.workspace.branch.title}
											</p>
										</div>
									</div>
									<p
										class="text-sm leading-relaxed {isDark
											? 'text-neutral-400'
											: 'text-neutral-600'}"
									>
										{content.proReady.workspace.branch.desc}
									</p>
									<div class="branch-rail mt-4">
										{#each content.proReady.workspace.branch.nodes as node, i}
											<div
												class="branch-node"
												style={`--delay:${i * 0.3}s`}
											>
												<span
													class="branch-pill font-mono border {isDark
														? 'bg-neutral-950/80 border-neutral-800 text-neutral-300'
														: 'bg-white border-neutral-200 text-neutral-700'}"
												>
													{node}
												</span>
											</div>
										{/each}
									</div>
								</div>

								<div
									class="workspace-panel rounded-3xl border p-5 transition-colors {isDark
										? 'bg-neutral-900/60 border-neutral-800'
										: 'bg-neutral-50 border-neutral-200'}"
								>
									<div class="flex items-center gap-3 mb-3">
										<div
											class="w-9 h-9 rounded-2xl flex items-center justify-center {isDark
												? 'bg-blue-500/10 text-blue-300'
												: 'bg-blue-50 text-blue-600'}"
										>
											<FolderTree class="w-4 h-4" />
										</div>
										<div class="space-y-0.5">
											<p
												class="text-xs font-bold uppercase tracking-[0.2em] text-blue-500"
											>
												{content.proReady.workspace.folder.title}
											</p>
										</div>
									</div>
									<p
										class="text-sm leading-relaxed {isDark
											? 'text-neutral-400'
											: 'text-neutral-600'}"
									>
										{content.proReady.workspace.folder.desc}
									</p>
									<div
										class="folder-tree mt-4 border rounded-2xl p-4 font-mono text-xs {isDark
											? 'bg-neutral-950/80 border-neutral-800 text-neutral-300'
											: 'bg-white border-neutral-200 text-neutral-700'}"
									>
										{#each content.proReady.workspace.folder.tree as row}
											<div
												class="tree-row"
												style={`--depth:${row.depth}`}
											>
												<span class="tree-dot"></span>
												<span>{row.label}</span>
											</div>
										{/each}
									</div>
								</div>
							</div>

							<div class="mt-6 flex flex-wrap gap-2">
								{#each content.proReady.workspace.badges as badge}
									<span
										class="px-3 py-1 rounded-full border text-[11px] font-bold uppercase tracking-wide {isDark
											? 'bg-neutral-950 border-neutral-800 text-neutral-400'
											: 'bg-neutral-50 border-neutral-200 text-neutral-600'}"
									>
										{badge}
									</span>
								{/each}
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Deployment -->
	<section
		class="py-24 transition-colors duration-300 {isDark
			? 'bg-neutral-950'
			: 'bg-white'}"
	>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="text-center mb-12 reveal">
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.deployment.title}
				</h2>
				<p
					class="max-w-2xl mx-auto {isDark
						? 'text-neutral-400'
						: 'text-neutral-600'}"
				>
					{content.deployment.desc}
				</p>
			</div>
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				{#each content.deployment.options as opt, i}
					<div
						class="p-8 rounded-3xl border transition-all reveal {isDark
							? 'bg-neutral-800/50 border-neutral-700 hover:bg-neutral-800'
							: 'bg-neutral-50 border-neutral-100 hover:bg-white hover:border-blue-200 shadow-sm hover:shadow-xl'}"
						style={`transition-delay:${i * 120}ms`}
					>
						<div class="flex items-center justify-between mb-4">
							<h3 class="text-2xl font-black">{opt.title}</h3>
							<span
								class="px-3 py-1 text-xs font-bold uppercase tracking-wide rounded-full {isDark
									? 'bg-blue-500/20 text-blue-400'
									: 'bg-blue-50 text-blue-700'}"
							>
								{lang === "ko" ? "배포" : "Deploy"}
							</span>
						</div>
						<p
							class="mb-4 leading-relaxed {isDark
								? 'text-neutral-400'
								: 'text-neutral-700'}"
						>
							{opt.description}
						</p>
						<div class="flex flex-wrap gap-2">
							{#each opt.badges as b}
								<span
									class="px-3 py-1 rounded-full border text-xs font-bold transition-colors {isDark
										? 'bg-neutral-900 border-neutral-700 text-neutral-400'
										: 'bg-white border-neutral-200 text-neutral-700'}"
								>
									{b}
								</span>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<section
		id="quickstart"
		class="py-24 transition-colors duration-300 overflow-hidden {isDark
			? 'bg-neutral-900'
			: 'bg-white'}"
	>
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="text-center mb-12 reveal">
				<h2 class="text-3xl sm:text-4xl font-black mb-4 tracking-tight">
					{content.quickstart.title}
				</h2>
				<p
					class="max-w-2xl mx-auto mb-8 {isDark
						? 'text-neutral-400'
						: 'text-neutral-600'}"
				>
					{content.quickstart.desc}
				</p>

				<!-- Tabs -->
				<div
					class="inline-flex p-1 rounded-xl mb-8 {isDark
						? 'bg-neutral-800'
						: 'bg-neutral-100'}"
				>
					<button
						onclick={() => (quickstartType = "docker")}
						class="px-6 py-2 rounded-lg text-sm font-bold transition-all {quickstartType ===
						'docker'
							? isDark
								? 'bg-neutral-700 text-white shadow-lg'
								: 'bg-white text-blue-600 shadow-sm'
							: 'text-neutral-500 hover:text-neutral-400'}"
					>
						{content.quickstart.tabs.docker}
					</button>
					<button
						onclick={() => (quickstartType = "podman")}
						class="px-6 py-2 rounded-lg text-sm font-bold transition-all {quickstartType ===
						'podman'
							? isDark
								? 'bg-neutral-700 text-white shadow-lg'
								: 'bg-white text-blue-600 shadow-sm'
							: 'text-neutral-500 hover:text-neutral-400'}"
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
									class="w-10 h-10 rounded-full flex items-center justify-center font-bold text-lg shadow-lg transition-transform group-hover:scale-110 {isDark
										? 'bg-blue-500 text-white shadow-blue-500/20'
										: 'bg-blue-600 text-white shadow-blue-500/30'}"
								>
									{i + 1}
								</div>
								{#if i < content.quickstart.steps.length - 1}
									<div
										class="w-0.5 h-full mt-4 min-h-[4rem] {isDark
											? 'bg-neutral-800'
											: 'bg-neutral-100'}"
									></div>
								{/if}
							</div>
							<div class="flex-1 pt-1 w-full">
								<h3 class="text-xl font-bold mb-4">
									{step.title}
								</h3>
								{#if step.code}
									<div
										class="rounded-2xl p-6 font-mono text-sm shadow-xl border relative overflow-hidden group/code {isDark
											? 'bg-neutral-950 border-neutral-800 text-neutral-400'
											: 'bg-neutral-900 border-neutral-800 text-neutral-300'}"
									>
										<div class="flex gap-1.5 mb-3">
											<div
												class="w-2.5 h-2.5 rounded-full {isDark
													? 'bg-neutral-800'
													: 'bg-neutral-700'}"
											></div>
											<div
												class="w-2.5 h-2.5 rounded-full {isDark
													? 'bg-neutral-800'
													: 'bg-neutral-700'}"
											></div>
											<div
												class="w-2.5 h-2.5 rounded-full {isDark
													? 'bg-neutral-800'
													: 'bg-neutral-700'}"
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
										class="p-6 rounded-2xl border font-medium {isDark
											? 'bg-blue-500/10 border-blue-500/20 text-blue-300'
											: 'bg-blue-50 border-blue-100 text-blue-900'}"
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

	<section
		class="py-24 transition-colors duration-300 relative overflow-hidden {isDark
			? 'bg-neutral-900 text-white'
			: 'bg-neutral-950 text-white'}"
	>
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
	<footer
		class="transition-colors duration-300 border-t {isDark
			? 'bg-neutral-950 border-neutral-800'
			: 'bg-white border-neutral-100'} py-16"
	>
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
				<p
					class="text-sm font-medium {isDark
						? 'text-neutral-500'
						: 'text-neutral-500'}"
				>
					© 2025 Open Codelabs Project. Released under MIT License.
				</p>
				<div class="flex gap-6">
					<a
						href="https://github.com/jaichangpark/open-codelabs"
						class="transition-all hover:scale-110 {isDark
							? 'text-neutral-500 hover:text-white'
							: 'text-neutral-400 hover:text-blue-600'}"
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

	.workflow-card {
		isolation: isolate;
	}

	.workflow-sheen {
		position: absolute;
		inset: -40% -30%;
		background:
			radial-gradient(circle at 20% 30%, rgba(59, 130, 246, 0.35), transparent 55%),
			radial-gradient(circle at 80% 20%, rgba(99, 102, 241, 0.3), transparent 50%),
			linear-gradient(120deg, rgba(59, 130, 246, 0.08), rgba(99, 102, 241, 0.12), rgba(59, 130, 246, 0.04));
		opacity: 0.7;
		z-index: 0;
		animation: workflow-sheen 10s ease-in-out infinite;
		pointer-events: none;
	}

	.workflow-steps {
		position: relative;
		z-index: 1;
	}

	.workflow-rail {
		position: absolute;
		left: 20px;
		top: 20px;
		bottom: 20px;
		width: 2px;
		background: linear-gradient(
			180deg,
			rgba(59, 130, 246, 0.05),
			rgba(59, 130, 246, 0.4),
			rgba(99, 102, 241, 0.6),
			rgba(59, 130, 246, 0.1)
		);
		border-radius: 9999px;
		opacity: 0.8;
	}

	.workflow-rail::after {
		content: "";
		position: absolute;
		left: -1px;
		width: 4px;
		height: 30%;
		border-radius: 9999px;
		background: linear-gradient(
			180deg,
			transparent,
			rgba(59, 130, 246, 0.9),
			rgba(99, 102, 241, 0.9),
			transparent
		);
		animation: workflow-flow 3.8s ease-in-out infinite;
	}

	.workflow-step {
		position: relative;
		padding-left: 44px;
	}

	.workflow-dot {
		position: absolute;
		left: 14px;
		top: 20px;
		width: 12px;
		height: 12px;
		border-radius: 9999px;
		background: #60a5fa;
		box-shadow: 0 0 0 6px rgba(59, 130, 246, 0.12);
		animation: workflow-pulse 2.6s ease-in-out infinite;
		animation-delay: var(--delay);
	}

	.prep-flow {
		position: relative;
		padding-left: 36px;
	}

	.prep-rail {
		position: absolute;
		left: 16px;
		top: 12px;
		bottom: 12px;
		width: 2px;
		background: linear-gradient(
			180deg,
			rgba(59, 130, 246, 0.15),
			rgba(16, 185, 129, 0.5),
			rgba(59, 130, 246, 0.2)
		);
		border-radius: 9999px;
		opacity: 0.9;
	}

	.prep-rail::after {
		content: "";
		position: absolute;
		left: -1px;
		width: 4px;
		height: 30%;
		border-radius: 9999px;
		background: linear-gradient(
			180deg,
			transparent,
			rgba(56, 189, 248, 0.9),
			transparent
		);
		animation: prep-flow 3.6s ease-in-out infinite;
	}

	.prep-step {
		position: relative;
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 12px;
		align-items: start;
		padding-left: 12px;
	}

	.prep-step-icon {
		width: 34px;
		height: 34px;
		flex-shrink: 0;
	}

	.prep-dot {
		position: absolute;
		left: 8px;
		top: 14px;
		width: 10px;
		height: 10px;
		border-radius: 9999px;
		background: #38bdf8;
		box-shadow: 0 0 0 6px rgba(56, 189, 248, 0.12);
		animation: prep-pulse 2.8s ease-in-out infinite;
		animation-delay: var(--delay);
	}

	.branch-rail {
		position: relative;
		padding-left: 18px;
		margin-top: 12px;
	}

	.branch-rail::before {
		content: "";
		position: absolute;
		left: 6px;
		top: 4px;
		bottom: 4px;
		width: 2px;
		border-radius: 9999px;
		background: linear-gradient(
			180deg,
			rgba(59, 130, 246, 0.1),
			rgba(59, 130, 246, 0.6),
			rgba(59, 130, 246, 0.1)
		);
	}

	.branch-rail::after {
		content: "";
		position: absolute;
		left: 5px;
		width: 4px;
		height: 34%;
		border-radius: 9999px;
		background: linear-gradient(
			180deg,
			transparent,
			rgba(56, 189, 248, 0.9),
			transparent
		);
		animation: branch-flow 3.4s ease-in-out infinite;
	}

	.branch-node {
		position: relative;
		padding-left: 14px;
		margin-bottom: 10px;
	}

	.branch-node::before {
		content: "";
		position: absolute;
		left: -2px;
		top: 10px;
		width: 10px;
		height: 10px;
		border-radius: 9999px;
		background: #60a5fa;
		box-shadow: 0 0 0 6px rgba(59, 130, 246, 0.1);
		animation: branch-pulse 2.6s ease-in-out infinite;
		animation-delay: var(--delay);
	}

	.branch-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border-radius: 9999px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.04em;
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
			"Liberation Mono", "Courier New", monospace;
	}

	.folder-tree {
		position: relative;
		overflow: hidden;
	}

	.folder-tree::after {
		content: "";
		position: absolute;
		inset: -40% 0;
		background: linear-gradient(
			120deg,
			transparent,
			rgba(59, 130, 246, 0.15),
			transparent
		);
		opacity: 0.6;
		animation: tree-scan 4.6s ease-in-out infinite;
		pointer-events: none;
	}

	.tree-row {
		position: relative;
		display: flex;
		align-items: center;
		gap: 8px;
		padding-left: calc(var(--depth) * 14px + 8px);
		margin-bottom: 6px;
	}

	.tree-row::before {
		content: "";
		position: absolute;
		left: calc(var(--depth) * 14px);
		top: 50%;
		width: 8px;
		height: 1px;
		background: rgba(148, 163, 184, 0.6);
	}

	.tree-dot {
		width: 6px;
		height: 6px;
		border-radius: 9999px;
		background: #34d399;
		box-shadow: 0 0 0 4px rgba(52, 211, 153, 0.15);
		flex-shrink: 0;
	}

	@keyframes workflow-sheen {
		0% {
			transform: translate3d(-6%, -4%, 0) scale(1);
			opacity: 0.55;
		}
		50% {
			transform: translate3d(6%, 6%, 0) scale(1.05);
			opacity: 0.9;
		}
		100% {
			transform: translate3d(-6%, -4%, 0) scale(1);
			opacity: 0.55;
		}
	}

	@keyframes workflow-flow {
		0% {
			transform: translateY(-40%);
			opacity: 0;
		}
		25% {
			opacity: 1;
		}
		75% {
			opacity: 1;
		}
		100% {
			transform: translateY(140%);
			opacity: 0;
		}
	}

	@keyframes workflow-pulse {
		0%,
		100% {
			transform: scale(1);
			box-shadow: 0 0 0 6px rgba(59, 130, 246, 0.12);
		}
		50% {
			transform: scale(1.15);
			box-shadow: 0 0 0 12px rgba(59, 130, 246, 0.08);
		}
	}

	@keyframes prep-flow {
		0% {
			transform: translateY(-40%);
			opacity: 0;
		}
		25% {
			opacity: 1;
		}
		75% {
			opacity: 1;
		}
		100% {
			transform: translateY(140%);
			opacity: 0;
		}
	}

	@keyframes prep-pulse {
		0%,
		100% {
			transform: scale(1);
			box-shadow: 0 0 0 6px rgba(56, 189, 248, 0.12);
		}
		50% {
			transform: scale(1.15);
			box-shadow: 0 0 0 12px rgba(56, 189, 248, 0.08);
		}
	}

	@keyframes branch-flow {
		0% {
			transform: translateY(-35%);
			opacity: 0;
		}
		25% {
			opacity: 1;
		}
		75% {
			opacity: 1;
		}
		100% {
			transform: translateY(140%);
			opacity: 0;
		}
	}

	@keyframes branch-pulse {
		0%,
		100% {
			transform: scale(1);
			box-shadow: 0 0 0 6px rgba(59, 130, 246, 0.1);
		}
		50% {
			transform: scale(1.15);
			box-shadow: 0 0 0 12px rgba(59, 130, 246, 0.08);
		}
	}

	@keyframes tree-scan {
		0% {
			transform: translate3d(-25%, -10%, 0);
			opacity: 0;
		}
		35% {
			opacity: 0.6;
		}
		70% {
			opacity: 0.4;
		}
		100% {
			transform: translate3d(25%, 10%, 0);
			opacity: 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.workflow-sheen,
		.workflow-rail::after,
		.workflow-dot,
		.prep-rail::after,
		.prep-dot,
		.branch-rail::after,
		.branch-node::before,
		.folder-tree::after {
			animation: none;
		}
	}
</style>
