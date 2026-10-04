CREATE TABLE `matches` (
	`id` text PRIMARY KEY NOT NULL,
	`game` text NOT NULL,
	`host` text NOT NULL,
	`guest` text,
	`status` text NOT NULL,
	`board` text NOT NULL,
	`turn` integer NOT NULL,
	`winner` integer,
	`moves` integer DEFAULT 0 NOT NULL,
	`revision` integer DEFAULT 0 NOT NULL,
	`created` integer NOT NULL,
	`updated` integer NOT NULL,
	`deadline` integer NOT NULL,
	`last_action` text,
	`settled` integer DEFAULT 0 NOT NULL,
	FOREIGN KEY (`host`) REFERENCES `players`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`guest`) REFERENCES `players`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE INDEX `matches_status_created_idx` ON `matches` (`status`,`created`);--> statement-breakpoint
CREATE INDEX `matches_host_status_idx` ON `matches` (`host`,`status`);--> statement-breakpoint
CREATE INDEX `matches_guest_status_idx` ON `matches` (`guest`,`status`);--> statement-breakpoint
CREATE TABLE `messages` (
	`id` text PRIMARY KEY NOT NULL,
	`player` text NOT NULL,
	`body` text NOT NULL,
	`created` integer NOT NULL,
	`deleted` integer DEFAULT 0 NOT NULL,
	FOREIGN KEY (`player`) REFERENCES `players`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE INDEX `messages_created_idx` ON `messages` (`created`);--> statement-breakpoint
CREATE TABLE `players` (
	`id` text PRIMARY KEY NOT NULL,
	`alias` text NOT NULL,
	`qor_id` text,
	`created` integer NOT NULL,
	`seen` integer NOT NULL,
	`last_chat` integer DEFAULT 0 NOT NULL,
	`chat_gate` text
);
--> statement-breakpoint
CREATE INDEX `players_seen_idx` ON `players` (`seen`);--> statement-breakpoint
CREATE UNIQUE INDEX `players_qor_idx` ON `players` (`qor_id`);--> statement-breakpoint
CREATE TABLE `reports` (
	`message` text NOT NULL,
	`player` text NOT NULL,
	`reason` text NOT NULL,
	`created` integer NOT NULL,
	PRIMARY KEY(`message`, `player`),
	FOREIGN KEY (`message`) REFERENCES `messages`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`player`) REFERENCES `players`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `standings` (
	`player` text NOT NULL,
	`game` text NOT NULL,
	`wins` integer DEFAULT 0 NOT NULL,
	`losses` integer DEFAULT 0 NOT NULL,
	`draws` integer DEFAULT 0 NOT NULL,
	`points` integer DEFAULT 0 NOT NULL,
	`updated` integer NOT NULL,
	PRIMARY KEY(`player`, `game`),
	FOREIGN KEY (`player`) REFERENCES `players`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE INDEX `standings_game_points_idx` ON `standings` (`game`,`points`);