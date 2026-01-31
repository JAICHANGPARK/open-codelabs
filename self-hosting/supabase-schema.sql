create extension if not exists "pgcrypto";

create table if not exists codelabs (
    id uuid primary key default gen_random_uuid(),
    title text not null,
    description text not null,
    author text not null,
    is_public boolean not null default true,
    quiz_enabled boolean not null default false,
    require_quiz boolean not null default false,
    require_feedback boolean not null default false,
    guide_markdown text,
    owner_id uuid references auth.users(id),
    created_at timestamptz not null default now()
);

create table if not exists steps (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    step_number integer not null,
    title text not null,
    content_markdown text not null
);

create table if not exists attendees (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    name text not null,
    code text not null,
    email text,
    current_step integer default 1,
    is_completed boolean default false,
    completed_at timestamptz,
    created_at timestamptz default now()
);

create table if not exists help_requests (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    attendee_id uuid not null references attendees(id) on delete cascade,
    step_number integer not null,
    status text not null default 'pending',
    created_at timestamptz default now()
);

create table if not exists chat_messages (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    sender_name text not null,
    message text not null,
    msg_type text not null default 'chat',
    target_id uuid,
    created_at timestamptz default now()
);

create table if not exists feedback (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    attendee_id uuid references attendees(id) on delete set null,
    difficulty text not null,
    satisfaction text not null,
    comment text,
    created_at timestamptz default now()
);

create table if not exists materials (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    title text not null,
    material_type text not null,
    link_url text,
    file_path text,
    created_at timestamptz default now()
);

create table if not exists quizzes (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    question text not null,
    quiz_type text not null default 'multiple_choice',
    options text not null,
    correct_answer integer not null,
    created_at timestamptz default now()
);

create table if not exists quiz_submissions (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    attendee_id uuid not null references attendees(id) on delete cascade,
    quiz_id uuid not null references quizzes(id) on delete cascade,
    answer text not null,
    is_correct boolean not null,
    created_at timestamptz default now()
);

create table if not exists submissions (
    id uuid primary key default gen_random_uuid(),
    codelab_id uuid not null references codelabs(id) on delete cascade,
    attendee_id uuid not null references attendees(id) on delete cascade,
    file_path text not null,
    file_name text not null,
    file_size bigint not null,
    created_at timestamptz default now()
);

create table if not exists participations (
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null references auth.users(id) on delete cascade,
    codelab_id uuid not null references codelabs(id) on delete cascade,
    attendee_id uuid references attendees(id) on delete set null,
    joined_at timestamptz default now(),
    unique (user_id, codelab_id)
);

create index if not exists idx_steps_codelab on steps(codelab_id);
create index if not exists idx_attendees_codelab on attendees(codelab_id);
create unique index if not exists idx_attendees_codelab_name on attendees(codelab_id, name);
create index if not exists idx_help_requests_codelab on help_requests(codelab_id);
create index if not exists idx_chat_messages_codelab on chat_messages(codelab_id);
create unique index if not exists idx_feedback_codelab_attendee on feedback(codelab_id, attendee_id);
create index if not exists idx_materials_codelab on materials(codelab_id);
create index if not exists idx_quizzes_codelab on quizzes(codelab_id);
create index if not exists idx_quiz_submissions_codelab on quiz_submissions(codelab_id);
create index if not exists idx_submissions_codelab on submissions(codelab_id);
create index if not exists idx_participations_user on participations(user_id);

alter publication supabase_realtime add table chat_messages;
alter publication supabase_realtime add table help_requests;
alter publication supabase_realtime add table attendees;
