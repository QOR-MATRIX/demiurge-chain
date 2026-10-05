CREATE TABLE `qor_logins` (
	`state` text PRIMARY KEY NOT NULL,
	`verifier` text NOT NULL,
	`created` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `qor_logins_created_idx` ON `qor_logins` (`created`);--> statement-breakpoint
CREATE TABLE `qor_sessions` (
	`id` text PRIMARY KEY NOT NULL,
	`sub` text NOT NULL,
	`qor_id` text NOT NULL,
	`username` text NOT NULL,
	`chain_account` text,
	`access_token` text NOT NULL,
	`refresh_token` text NOT NULL,
	`created` integer NOT NULL,
	`expires` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `qor_sessions_expires_idx` ON `qor_sessions` (`expires`);--> statement-breakpoint
CREATE INDEX `qor_sessions_sub_idx` ON `qor_sessions` (`sub`);