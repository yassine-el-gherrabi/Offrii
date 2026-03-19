# Offrii Notification Strategy Analysis

**Date**: 2026-03-15
**Branch**: feature/frontend-redesign-v2
**Scope**: Full audit of existing notification infrastructure + strategic plan for new notifications

---

## Part 1: Current Notification Infrastructure

### What Already Exists (Fully Functional)

Offrii has a solid, production-ready push notification foundation:

**Backend Infrastructure**
- `ApnsNotificationService` -- Full APNs integration using the `a2` crate with token-based auth (P8 key)
- Supports both sandbox and production APNs endpoints
- Batch sending with `MAX_CONCURRENT_SENDS = 10` concurrent requests
- Invalid token detection and cleanup (handles `BadDeviceToken` / `Unregistered`)
- Token redaction in logs for security

**Push Token Management**
- `PushToken` model: `id`, `user_id`, `token`, `platform`, `created_at`
- `PushTokenRepo` trait with `upsert`, `delete_by_token`, `find_by_user`
- REST endpoints: `POST /push-tokens` (register), `DELETE /push-tokens/{token}` (unregister)
- `PushTokenService` for registration/unregistration

**Notification Traits (Well-Abstracted)**
- `NotificationService` trait with `send_batch(&[NotificationRequest])` -- easy to mock/test
- `NotificationRequest` struct: `device_token`, `title`, `body`
- `NotificationOutcome` enum: `Sent`, `InvalidToken`, `Error(String)`

**Reminder Job System**
- `ReminderService` with `execute_hourly_tick()` -- runs every hour
- Distributed lock via Redis (`SET NX EX 300`) to prevent concurrent execution
- Per-user anti-spam keys with TTL based on frequency (`daily`/`weekly`/`monthly`)
- Scoring algorithm: `PRIORITY_WEIGHT * (4 - priority) + AGE_WEIGHT * days_waiting`
- Selects top 3 items older than 7 days for the reminder
- User preferences: `reminder_freq`, `reminder_time`, `timezone`, `utc_reminder_hour`
- Eligible user query: `find_eligible_for_reminder(utc_hour)`

**iOS App**
- `PushTokenService.swift` -- API client for registering/unregistering tokens
- `PostAuthSetupView` -- asks for notification permission during onboarding (`.alert`, `.badge`, `.sound`)
- `APIEndpoint.registerToken` / `.unregisterToken` defined

### Notifications Currently Being Sent

| Event | Who Receives | Title (FR) | Body |
|-------|-------------|------------|------|
| **Friend request sent** | Recipient | "Demande d'ami" | "{username} veut etre votre ami" |
| **Friend request accepted** | Sender | "Demande acceptee !" | "{username} a accepte votre demande d'ami" |
| **Wishlist reminder** | Item owner | "Tes envies t'attendent !" | "N'oublie pas : {item names}" |
| **Community wish closed** | Donor | "Souhait ferme" | "L'auteur a ferme son souhait." |
| **Community wish offer received** | Owner | "Offre d'aide !" | "{donor_name} propose de vous aider !" |
| **Community wish offer withdrawn** | Owner | "Offre retiree" | "L'offre d'aide a ete retiree." |
| **Community wish offer rejected** | Donor | "Offre declinee" | "L'auteur a decline votre offre d'aide." |
| **Community wish confirmed** | Donor | "Don confirme !" | "Votre don a ete confirme, merci !" |
| **Community wish reported** | Owner | "Souhait signale" | "Votre souhait a ete signale..." |
| **Community wish approved** | Owner | "Souhait approuve" | "Votre souhait a ete approuve..." |
| **Community wish rejected** | Owner | "Souhait refuse" | "Votre souhait n'a pas ete retenu..." |
| **Wish message received** | Other participant | Sender name | Message preview (50 chars) |
| **Circle: member joined** | Other members | "Nouveau membre !" | "{username} a rejoint le cercle" |
| **Circle: item shared** | Other members | "Nouvel article partage !" | "{item name} a ete partage dans le cercle" |

### Critical Gap: iOS Token Forwarding

The iOS app requests notification permission in `PostAuthSetupView` but **there is no `AppDelegate` or `UNUserNotificationCenterDelegate` to capture the device token and forward it to the backend**. The `PushTokenService.swift` exists but is never called with an actual device token from APNs. This means:

- Permission is requested but the token is never captured
- `UIApplication.shared.registerForRemoteNotifications()` is never called
- `didRegisterForRemoteNotificationsWithDeviceToken` is not implemented
- **No push notifications are actually being delivered to real devices**

This is the most critical gap to fix before any notification strategy matters.

### What Does NOT Have Notifications (Gaps)

| Event | Current Behavior | Notification? |
|-------|-----------------|---------------|
| **Item claimed (app)** | Circle event logged, no notification to owner | NO |
| **Item unclaimed (app)** | Circle event logged | NO |
| **Item claimed (web)** | No notification | NO |
| **Item claimed (share link)** | No notification | NO |
| **Share link viewed** | No tracking | NO |
| **Circle invite created** | No notification to invitee | NO |
| **Item added by friend** | No notification | NO |
| **Birthday/occasion reminders** | No birthday tracking | NO |
| **Price changes** | No price tracking | NO |
| **New circle created (direct)** | Created on friendship, no notification | NO |

---

## Part 2: Competitive Landscape -- What Do Wishlist Apps Notify About?

### Amazon Wishlist
- Price drop notifications (their killer feature)
- "Don't spoil my surprises" setting -- purchased items remain visible as unpurchased for several weeks
- Delivery notifications (can be routed to alternate email/phone)
- Back-in-stock alerts
- No notification when someone claims/buys from your list (by design -- surprise preservation)

### Elfster
- Email notification categories with granular opt-in/out:
  - "When friends add items to their Wishlist"
  - "When friends have a birthday"
  - "When someone likes or comments on your activity feed"
  - "Gift ideas and suggestions for your birthday"
  - "Reminders to update your Wishlist"
  - "Gift exchange anniversary reminders"
- Per-exchange notification settings (different settings for family vs. work exchanges)
- Mandatory notifications for active gift exchanges (can't opt out of organizer messages)

### Giftful
- Gift reservation/claiming system with surprise preservation
- All profiles and lists public by default (privacy concern flagged by security researchers)
- Follow model without approval required

### Wishlists (top-rated app 2025)
- Universal store compatibility
- Privacy-focused (no data selling)
- Gift reservation to prevent duplicates
- Easy sharing via link, email, social media

### Favory
- "Privacy-first" positioning
- Anonymous claiming system -- key differentiator

### Key Industry Patterns
1. **Surprise preservation is the #1 design constraint** -- never tell the list owner what was claimed/bought
2. **Gift reservation/claiming** prevents duplicate purchases among gifters
3. **Granular notification preferences** -- users want per-category control
4. **Birthday/occasion reminders** are table-stakes
5. **Wishlist update notifications** ("your friend added new items") drive engagement
6. **Price drop/back-in-stock** is high-value but requires infrastructure

---

## Part 3: Push Notification Best Practices 2025-2026

### Key Statistics
- iOS opt-in rate: ~51% (vs. Android ~81%)
- iOS CTR: ~3.4%
- Personalized notifications: 400% improvement in reaction rates
- Tailored send times: 40% improvement
- Rich formats (images, buttons): 25% improvement
- Emojis: ~20% improvement
- Average US smartphone user: 46 push notifications/day

### Critical Best Practices for Offrii

1. **Pre-permission priming**: Show value before the system prompt. Offrii already does this in PostAuthSetupView -- good.

2. **Transactional vs. promotional separation**: Never disguise marketing as utility. Users mentally separate "my item was claimed" from "check out what's trending."

3. **User control is non-negotiable**: A single "Allow notifications?" toggle is no longer acceptable UX in 2026. Users want:
   - What they're notified about (per-category)
   - How often
   - How messages are delivered

4. **Frequency discipline**: More than 3-5 push notifications per week from a single app leads to opt-outs. For a wishlist app, 1-3/week is the sweet spot.

5. **Timing**: Respect timezone (Offrii already stores `timezone` and `utc_reminder_hour` -- well designed). Never send between 10pm-8am local time.

6. **Deep linking**: Every notification should take the user to the exact relevant screen.

7. **Rich content**: Use images when possible (item images for claim notifications, avatar for friend notifications).

---

## Part 4: Notification Classification and Prioritization

### TIER 1 -- HIGH VALUE (Users actively want these)

These are notifications users would be disappointed NOT to receive. They provide clear, immediate value.

#### 1. Item Claimed Notification (to GIFTERS in the circle, NOT the owner)
- **Trigger**: Someone claims an item in a shared circle
- **Recipient**: Other circle members who can see the item (NOT the item owner)
- **Why high value**: Prevents duplicate gift purchases -- the core value prop
- **Privacy**: NEVER notify the item owner that their item was claimed (spoils surprise)
- **Message**: "Un cadeau reserve !" / "{claimer} a reserve un article dans {circle_name}"
- **Implementation**: Modify `on_item_claimed` in circle_service to send notifications to members except owner and claimer

#### 2. Friend Request Received
- **Status**: ALREADY IMPLEMENTED
- **Assessment**: Good. Keep as-is.

#### 3. Friend Request Accepted
- **Status**: ALREADY IMPLEMENTED
- **Assessment**: Good. Keep as-is.

#### 4. New Message in Community Wish
- **Status**: ALREADY IMPLEMENTED
- **Assessment**: Good. Message preview is capped at 50 chars -- appropriate.

#### 5. Community Wish Offer Received
- **Status**: ALREADY IMPLEMENTED
- **Assessment**: Good. Critical for the community wish matching flow.

#### 6. Wishlist Reminder (Nudge to Share)
- **Status**: PARTIALLY IMPLEMENTED -- reminders exist but only say "don't forget"
- **Enhancement**: Instead of just reminding about old items, nudge users to share items with circles
- **Message**: "Tu as 5 envies non partagees -- partage-les avec tes proches !"
- **Why high value**: Drives the core sharing behavior that makes the app useful

#### 7. Item Web-Claimed via Share Link
- **Trigger**: Anonymous user claims an item through a public share link
- **Recipient**: Item owner
- **Why high value**: Owner needs to know someone is getting them a gift (without knowing who, just that the item is "reserved")
- **Privacy consideration**: Show "Quelqu'un a reserve [item name]" -- do NOT show the claimer's name
- **Wait**: Actually, this DOES spoil the surprise. See "Notifications to AVOID" section.

### TIER 2 -- MEDIUM VALUE (Nice to have, drives engagement)

#### 8. New Item Shared in Circle
- **Status**: ALREADY IMPLEMENTED
- **Assessment**: Good. Drives discovery.

#### 9. Someone Joined Your Circle
- **Status**: ALREADY IMPLEMENTED
- **Assessment**: Good.

#### 10. Friend Added New Items to Their Wishlist
- **Trigger**: A friend adds new items (batch -- don't notify per item)
- **Recipient**: Their friends (people who have a friendship)
- **Why medium value**: Helps friends discover what to get. Elfster offers this.
- **Frequency control**: Batch -- at most once per day per friend. "Marie a ajoute 3 nouvelles envies !"
- **Privacy**: Only notify about non-private items

#### 11. Circle Item Unclaimed
- **Trigger**: Someone unclaims an item in a circle
- **Recipient**: Other circle members (NOT the owner)
- **Why medium value**: Alerts other potential gifters that the item is available again
- **Message**: "Un article est de nouveau disponible dans {circle_name}"

#### 12. Birthday/Occasion Approaching
- **Trigger**: X days before a friend's birthday (requires birthday field on user profile)
- **Recipient**: Their friends
- **Why medium value**: Drives gift-giving behavior at the right time
- **Currently missing**: No birthday field exists in the User model
- **Message**: "L'anniversaire de {name} est dans {N} jours -- consulte sa liste !"

#### 13. Community Wish Status Updates
- **Status**: ALREADY IMPLEMENTED (confirmed, rejected, approved, etc.)
- **Assessment**: Good coverage of the full lifecycle.

### TIER 3 -- LOW VALUE (Implement later, with caution)

#### 14. Share Link Activity Summary
- **Trigger**: Weekly digest of share link views/claims
- **Recipient**: Share link owner
- **Why low value**: Interesting but not actionable. Better as in-app analytics.

#### 15. Circle Feed Digest
- **Trigger**: Weekly summary of circle activity
- **Recipient**: Circle members
- **Why low value**: Could be useful for low-activity users but risks being noise

#### 16. Re-engagement Nudge
- **Trigger**: User hasn't opened app in 14+ days
- **Recipient**: Inactive users
- **Why low value**: Generic re-engagement has low CTR. Only valuable if personalized ("Tu as 3 envies qui attendent depuis 30 jours")

#### 17. Item Approaching Event Date
- **Trigger**: If items or circles are associated with events/dates
- **Recipient**: Circle members
- **Why low value**: Requires event/date infrastructure that doesn't exist yet

---

## Part 5: Notifications to AVOID

### 1. NEVER Notify the Item Owner About Claims
- **Why**: This is the cardinal sin of wishlist apps. If you tell someone "your item was claimed," you spoil the surprise.
- **Exception**: Web claims via share links where the owner explicitly shared the link publicly. Even then, use vague language: "Quelqu'un a reserve un de tes articles" without saying which one.
- **Amazon's approach**: "Don't spoil my surprises" keeps purchased items shown as unpurchased for weeks.

### 2. NEVER Notify About Who Claimed What
- **Why**: Even telling other circle members "Marie a reserve le velo" could leak back to the owner.
- **Better approach**: Keep claim notifications vague: "Un article a ete reserve dans {circle}" -- don't name the item or the claimer to anyone who might tell the owner.

### 3. NEVER Send Promotional/Marketing Push Notifications
- **Why**: Offrii is a utility app, not a shopping platform. Any "check out trending wishes!" or "discover new features!" notifications will trigger opt-outs fast.
- **Rule**: Every notification must be triggered by a human action relevant to the recipient.

### 4. AVOID Excessive Frequency
- **Rule**: Maximum 1 notification per event type per day. Batch when possible.
- **Example**: If a friend adds 5 items in one session, send ONE notification: "Marie a ajoute 5 nouvelles envies"
- **Never**: 5 separate notifications for 5 items.

### 5. AVOID Notifying About Declined/Cancelled Actions
- Friend request declined: Do NOT notify the sender (feels like rejection)
- **Currently correct**: Offrii doesn't notify on decline. Keep it this way.

### 6. AVOID Notifying About Your Own Actions
- This seems obvious but: never send "You just added an item to your wishlist" as a push notification.
- Offrii's current implementation correctly excludes the acting user in `notify_members(circle_id, exclude_user, ...)`.

### 7. AVOID Notification Content That Reveals Private Information
- Items marked `is_private: true` should NEVER appear in any notification to anyone
- Claim notifications should NEVER include the item name if the notification goes to anyone other than the claimer themselves

---

## Part 6: Prioritized Implementation Plan

### Phase 0 -- Fix Critical Infrastructure (BLOCKER)
**Priority**: Must-do before anything else

1. **Add AppDelegate/UNUserNotificationCenterDelegate to iOS app**
   - Implement `didRegisterForRemoteNotificationsWithDeviceToken`
   - Call `UIApplication.shared.registerForRemoteNotifications()` after permission grant
   - Convert device token data to hex string
   - Call `PushTokenService.shared.registerToken(token:)` to send to backend
   - Handle `didFailToRegisterForRemoteNotificationsWithError`
   - Implement `userNotificationCenter(_:didReceive:)` for tap handling

2. **Add notification handling for foreground state**
   - Implement `userNotificationCenter(_:willPresent:)` to show banners when app is in foreground
   - Consider suppressing notifications for the screen the user is already viewing

3. **Add deep linking from notifications**
   - Include `category` and `data` payload in APNs messages (currently only `title`/`body`)
   - Extend `NotificationRequest` struct to include optional JSON payload
   - Route notification taps to the correct screen in the app

### Phase 1 -- High-Value New Notifications
**Priority**: First sprint after Phase 0

4. **Item Claimed notification to circle members (not owner)**
   - Modify `CircleService::on_item_claimed` to notify members except owner and claimer
   - Message: "Un article a ete reserve dans {circle_name}" (vague, no item name)
   - This is the single most valuable missing notification

5. **Item Unclaimed notification to circle members (not owner)**
   - Modify `CircleService::on_item_unclaimed` similarly
   - Message: "Un article est de nouveau disponible dans {circle_name}"

6. **Web claim notification to item owner (carefully worded)**
   - When someone claims via a share link, notify owner
   - Message: "Quelqu'un s'interesse a ta liste !" (vague -- no item name, no claimer name)
   - This tells the owner their share link is working without spoiling surprises

### Phase 2 -- Notification Preferences
**Priority**: Before scaling notification volume

7. **Add notification_preferences to User model**
   ```
   notification_preferences JSONB DEFAULT '{}'
   ```
   Categories:
   - `friend_requests`: bool (default true)
   - `circle_activity`: bool (default true)
   - `item_claims`: bool (default true)
   - `reminders`: bool (default true)
   - `community_wishes`: bool (default true)
   - `friend_updates`: bool (default false -- opt-in)

8. **Add Notification Settings screen in iOS app**
   - Toggle per category
   - Link from Settings/Profile screen

9. **Backend: Check preferences before sending**
   - Add preference check in each `notify_user` / `notify_members` call
   - Respect user opt-outs per category

### Phase 3 -- Engagement Notifications
**Priority**: After core notifications are solid

10. **Friend wishlist updates** (batched daily)
    - Batch job: "Marie a ajoute 3 nouvelles envies"
    - Run as part of the existing hourly tick or as a separate daily job
    - Default: opted-out (users opt in)

11. **Enhanced reminders**
    - "Tu as {N} envies non partagees -- partage-les !"
    - Vary reminder messaging to avoid habituation

12. **Birthday reminders** (requires new infrastructure)
    - Add `birthday` field to User model
    - Add birthday entry to profile setup flow
    - Job: check daily, notify friends N days before

### Phase 4 -- Advanced Features
**Priority**: Future roadmap

13. **Notification grouping/threading** (iOS notification groups)
14. **Rich notifications** with item images
15. **Action buttons** ("Voir" / "Ignorer" directly on notification)
16. **Notification history/inbox** in-app
17. **FCM support** for Android (when/if Android app is built)

---

## Part 7: Technical Recommendations

### Extend NotificationRequest for Deep Linking
```rust
pub struct NotificationRequest {
    pub device_token: String,
    pub title: String,
    pub body: String,
    pub category: Option<String>,      // iOS notification category for action buttons
    pub thread_id: Option<String>,      // iOS notification grouping
    pub custom_data: Option<serde_json::Value>,  // Deep link data
}
```

### Refactor notify_user Pattern
The `notify_user` helper is duplicated across 4 services (friend_service, wish_message_service, community_wish_service, circle_service -- the `notify_members` variant). Consider extracting to a shared utility:

```rust
// services/notification_helpers.rs
pub fn notify_user(
    push_token_repo: Arc<dyn PushTokenRepo>,
    notification_svc: Arc<dyn NotificationService>,
    user_id: Uuid,
    title: String,
    body: String,
) { ... }

pub fn notify_users(
    push_token_repo: Arc<dyn PushTokenRepo>,
    notification_svc: Arc<dyn NotificationService>,
    user_ids: Vec<Uuid>,
    title: String,
    body: String,
) { ... }
```

### Add Notification Preference Checking
```rust
pub async fn should_notify(
    user_repo: &dyn UserRepo,
    user_id: Uuid,
    category: &str,
) -> bool { ... }
```

### Batching Strategy for Friend Updates
Rather than real-time notifications for "friend added items," use a batch approach:
- Track new items added per user per day in Redis: `new_items:{user_id}:{date}` (increment)
- In the hourly tick (or a separate daily job), check friends and send batched updates
- This prevents notification storms when someone adds 10 items at once

### Localization
Current notifications are hardcoded in French. The User model has a `locale` field. Use it:
- Store notification templates with locale keys
- Pass `locale` into the notification building logic
- Start with `fr` and `en`

---

## Summary

### What Offrii Does Well
- Solid APNs infrastructure with the `a2` crate
- Good abstraction via traits (easy to test and extend)
- Reminder system with anti-spam, scoring, timezone awareness
- Notifications for friend requests, community wishes, circle activity
- User-configurable reminder frequency and time

### Critical Gaps (in priority order)
1. **iOS app never captures or forwards the device token** -- no notifications actually arrive
2. **No claim notifications to circle members** -- the single most valuable missing notification
3. **No notification preferences** -- users can't control what they receive
4. **No deep linking** -- notifications open the app but don't navigate to relevant content
5. **Duplicated `notify_user` pattern** across 4 services
6. **No notification payload beyond title/body** -- no custom data for routing
7. **Hardcoded French** -- `locale` field exists but isn't used for notifications

### One-Line Takeaway
Fix the iOS token forwarding first, then add claim notifications for circle members (never the owner), and build notification preferences before scaling volume.
