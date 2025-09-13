

# **Functional Specification of the Campfire Chat Application**

## **Part I: Foundational Concepts & System-Wide Requirements**

This document provides a comprehensive functional specification for the Campfire chat application. It is intended to serve as a rigorous and exhaustive blueprint for development, detailing all user journeys, system behaviors, and underlying data structures. The requirements outlined herein are derived from a deep analysis of the application's codebase, database schema, and system tests.

### **Section 1: The Account Ecosystem**

The Campfire application is architected around a central, singular account that serves as the container for all users, rooms, and data. This foundational concept dictates the application's security model, user onboarding process, and overall administrative structure.

#### **1.1. Single-Tenancy Model**

The application operates on a strict single-tenant architecture. For any given deployment, a single Account record serves as the root entity for the entire system.1 This design is explicitly stated in the application's documentation, which notes, "Campfire is single-tenant: any rooms designated 'public' will be accessible by all users in the system".1 To support entirely distinct groups of users, separate instances of the application must be deployed.1 This architectural choice simplifies the data model by eliminating the need for cross-tenant data segregation and complex permission checks, focusing instead on the interactions within one cohesive organization or community.

#### **1.2. Account Attributes**

The singular Account entity possesses several key attributes that define the identity and appearance of the application instance, as detailed in the database schema and Account model.1

* **Name:** The account must have a non-null string attribute for its name, which serves as the primary identifier for the chat instance (e.g., "37signals Campfire").  
* **Logo:** The account may have an associated logo image file. This is managed via Active Storage, allowing an administrator to upload a brand image for the instance.1  
* **Custom Styles:** The system supports instance-wide visual customization through a custom\_styles text attribute on the Account model. This allows an administrator to inject arbitrary CSS to override default application styles, providing a mechanism for branding and theming.1

#### **1.3. User Onboarding via Join Code**

Access to the application for new users is governed by a shared secret mechanism. The Account model includes a join\_code attribute, which is a mandatory, unique, and automatically generated alphanumeric string formatted as "XXXX-XXXX-XXXX".1 This code functions as the sole gateway for unauthenticated users to access the registration page. The

UsersController enforces this by executing a verify\_join\_code check before rendering the new user form or processing its creation.1

This design implies a high-trust environment. The security model is not built for a multi-customer Software-as-a-Service (SaaS) product but rather for a private community or organization where the join code can be distributed to trusted individuals. There is no user-specific invitation or administrator approval workflow for new sign-ups; possession of the valid join code is sufficient for registration. Administrators retain control over this access gate through the Accounts::JoinCodesController, which provides the functionality to regenerate the join\_code, thereby invalidating the previous one and securing the instance against further registrations using the old code.1

### **Section 2: User Roles and Permissions**

The application employs a role-based access control (RBAC) system to manage user permissions. This system is defined within the User model and is fundamental to securing administrative functions while providing a seamless experience for standard users and bots.

#### **2.1. Role Definitions**

The User model defines a role enum with three distinct levels of privilege 1:

* **Member (role: 0):** This is the default role assigned to all newly created users. Members possess baseline permissions necessary for participation in the chat application, including sending messages, joining and leaving accessible rooms, and managing their own profile.  
* **Administrator (role: 1):** This is a privileged role with extensive permissions to manage the entire application instance. Administrators can modify account settings, manage all users (including promoting other users to administrators), manage all rooms, and oversee bot integrations.  
* **Bot (role: 2):** This is a specialized, non-human role designed for automated integrations. Bots authenticate programmatically and have a highly restricted set of permissions, primarily focused on posting messages to rooms via webhooks.

#### **2.2. Authorization Logic**

Authorization is consistently enforced throughout the application via the ensure\_can\_administer method, which is typically invoked as a controller before\_action.1 This method relies on the

User\#can\_administer?(record) method to determine access rights.1

A critical aspect of this permission model is its nuanced "ownership" override. The can\_administer? check grants permission under two conditions: either the user has the administrator role, or the user is the creator of the specific record being acted upon. For example, the MessagesController protects its update and destroy actions with ensure\_can\_administer.1 This means a standard

Member who created a specific message is authorized to edit or delete that message, effectively granting them administrative control over their own content. This dual-condition logic—checking for role or ownership—is a core requirement for all destructive or sensitive actions within the system. It empowers users while maintaining a clear administrative hierarchy.

#### **2.3. Bot Permissions**

Bots are treated as a distinct user type with a unique authentication and authorization flow. They authenticate using a bot\_token rather than a password-based session.1 To prevent bots from accessing the standard user-facing interface, a

deny\_bots filter is applied globally in the Authentication concern.1 Bot access is only permitted on controllers that explicitly opt-in using

allow\_bot\_access, such as the Messages::ByBotsController, which provides a dedicated API endpoint for bots to post messages.1 This strict separation ensures that bots can only perform their designated programmatic functions.

#### **2.4. User Roles and Permissions Matrix**

The following table provides a structured overview of the permissions granted to each user role across the application's primary feature areas.

| Feature Area | Member | Administrator | Bot |
| :---- | :---- | :---- | :---- |
| **Account Settings** | Read-only (Name/Logo) | Create/Read/Update | N/A |
| **User Management** | N/A (Can manage own profile) | Create/Read/Update/Deactivate (All Users) | N/A |
| **Bot Management** | N/A | Create/Read/Update/Deactivate (All Bots) | N/A |
| **Room Creation** | Create (Direct) | Create (Open, Closed, Direct) | N/A |
| **Room Management** | Update/Delete (Own rooms) | Update/Delete (All rooms) | N/A |
| **Membership Management** | Manage own involvement | Add/Remove any user (Closed rooms) | N/A |
| **Message Actions** | Create/Read, Update/Delete (Own messages) | Create/Read, Update/Delete (All messages) | Create (via API) |
| **Message Boosting** | Create/Delete (Own boosts) | Create/Delete (Own boosts) | N/A |
| **Search** | Full access to reachable messages | Full access to reachable messages | N/A |
| **Authentication** | Email/Password | Email/Password | Bot Token |

### **Section 3: Room Architecture and Behavior**

Rooms are the primary containers for conversation in Campfire. The system's architecture supports different types of rooms with distinct behaviors regarding privacy, membership, and notifications, all managed through a flexible data model.

#### **3.1. Single Table Inheritance (STI)**

The Room model utilizes Single Table Inheritance (STI) to manage different room types within a single database table. A type column in the rooms table stores the class name of the specific room subclass, allowing for shared functionality in the base Room model while enabling specialized behavior in its descendants.1 The application defines three concrete room types:

Rooms::Open, Rooms::Closed, and Rooms::Direct.

#### **3.2. Room Types and Membership Rules**

Each room type has a unique set of rules governing user membership and accessibility 1:

* **Open Rooms (Rooms::Open):** These rooms are public within the account. All active users are automatically granted membership upon creation. Furthermore, any new user who joins the account is retroactively granted membership to all existing Open rooms, ensuring universal access.1  
* **Closed Rooms (Rooms::Closed):** These are private, invitation-only rooms. Membership must be explicitly granted by an administrator or the room's creator. The user interface for creating a Closed room must include a mechanism for selecting an initial list of members from all users in the account.1  
* **Direct Message Rooms (Rooms::Direct):** These rooms facilitate private conversations between a specific set of two or more users. The system enforces a singleton pattern for Direct rooms; for any given combination of users, only one Direct room can exist. If a user initiates a conversation with a set of users for whom a room already exists, the system must locate and reuse the existing room rather than creating a new one.1

#### **3.3. Membership and Notifications**

The link between a User and a Room is defined by the Membership model.1 This model is central to the application's real-time functionality, as it tracks not only who is in a room but also their notification preferences and connection status.

A user's notification settings for a given room are controlled by the involvement attribute on their Membership record, which is an enum with four possible states 1:

* **everything:** The user receives a push notification for every new message posted in the room. This is the default setting for Direct Message rooms.  
* **mentions:** The user receives a push notification only when they are explicitly @mentioned in a message. This is the default for Open and Closed rooms.  
* **nothing:** The user receives no push notifications from the room.  
* **invisible:** The user receives no notifications, and the room is hidden from their sidebar list.

The application's real-time presence and unread status logic is also managed at the Membership level. The Membership model contains connected\_at and connections attributes, managed by the Connectable concern, which track whether a user is actively connected to a specific room's real-time channel.1 A room is marked as "unread" for a user only when a new message arrives and that user's

Membership for that room is in a disconnected state.1 This granular, per-room presence tracking is a critical requirement, as it ensures that notifications and unread indicators are triggered accurately based on whether a user is actively viewing a conversation, not just whether they are online in the application globally.

#### **3.4. Room Type Comparison**

The following table summarizes the key behavioral differences between the three room types, which are enforced by their respective STI subclasses.

| Property | Open Room | Closed Room | Direct Message Room |
| :---- | :---- | :---- | :---- |
| **Accessibility** | Public to all account users | Private to invited members | Private to specified members |
| **Membership Granting** | Automatic for all active and new users | Manual invitation by admin/creator | Automatic for specified users |
| **Creation UI** | Requires only a room name | Requires a room name and a user selector | Requires only a user selector |
| **Naming Convention** | User-defined name | User-defined name | Name is auto-generated from member list |
| **Default Notification** | @mentions only | @mentions only | All messages (everything) |
| **Singleton Behavior** | No | No | Yes (for a given set of users) |

## **Part II: Core User Journeys**

This section details the primary workflows and interactions users will experience within the Campfire application. Each journey is broken down into a sequence of steps, outlining user actions and the corresponding system requirements and responses.

### **Section 4: Onboarding and Authentication Journey**

The onboarding process is bifurcated into two mutually exclusive paths: the initial setup of a new Campfire instance and the subsequent registration of new users to that existing instance.

#### **4.1. First Run Application Setup**

This journey occurs only once in the application's lifecycle, when it is deployed for the first time with an empty database.

* **Trigger:** A user accesses the application when no Account record exists in the database. The system must detect this state (Account.any? is false) and route the user accordingly.1  
* **Journey:**  
  1. The user is presented with the "Set up Campfire" screen, rendered by the FirstRunsController.1 This screen is accessible without authentication.  
  2. The user must provide a name, a valid email address, and a password for the first administrator account. An avatar may be provided optionally.  
  3. Upon submission, the system must perform a series of atomic operations, managed by the FirstRun model: create the singleton Account record, create the first User with the administrator role, and create an initial Rooms::Open named "All Talk".1  
  4. The system must automatically grant the new administrator membership to the "All Talk" room.  
  5. A new session must be created for the administrator, and they must be logged in and redirected to the main application interface, landing in the "All Talk" room.1  
  6. Once this first run is complete, the setup URL must become permanently inaccessible, redirecting any future requests to the application's root.1

#### **4.2. New User Registration**

Once the application has been initialized, all subsequent user onboarding occurs through a join code-gated registration process.

* **Pre-requisite:** The user must possess the valid, current join\_code for the account.  
* **Journey:**  
  1. The user navigates to the registration URL, which must include the join code (e.g., /join/:join\_code).  
  2. The system, via the UsersController, must validate that the provided join\_code matches the one stored on the Account record. Invalid codes must result in a "Not Found" error.1  
  3. The user is presented with a registration form to provide their name, email, password, and an optional avatar.  
  4. Upon submission, the system must create a new User record with the default member role.  
  5. If the provided email address already exists, the system must not create a new user but instead redirect to the sign-in page with the email pre-filled, indicating the user should log in instead.1  
  6. Upon successful creation, the new user must be automatically granted membership to all existing Rooms::Open in the system.1  
  7. A new session must be created for the user, logging them in and redirecting them to their last visited room or a default room.1

#### **4.3. User Sign-In and Sign-Out**

This journey describes the standard authentication flow for existing users.

* **Journey (Sign-In):**  
  1. An unauthenticated user navigates to the sign-in page.  
  2. The system must first perform a browser compatibility check. If the user's browser does not meet the minimum version requirements defined in the AllowBrowser concern, an "incompatible browser" page must be displayed instead of the login form.1  
  3. The user provides their email address and password.  
  4. The SessionsController\#create action must be rate-limited to prevent brute-force attacks, returning a "Too Many Requests" error if the threshold is exceeded.1  
  5. The system validates the credentials against active users. Upon success, a Session record is created, a secure, HttpOnly session\_token cookie is set, and the user is redirected to the application interior.1  
* **Journey (Sign-Out):**  
  1. A signed-in user initiates the sign-out process.  
  2. The SessionsController\#destroy action is invoked.  
  3. The system must perform two key actions: remove the push notification subscription associated with the current device (if any) from the database, and delete the session\_token cookie from the browser.1  
  4. The user is redirected to the sign-in page.

#### **4.4. Session Transfer (Cross-Device Login)**

This journey provides a secure, passwordless method for a user to log into a new device.

* **Journey:**  
  1. A logged-in user navigates to their user profile page.  
  2. The system must generate and display a unique, single-use session transfer URL and a corresponding QR code. This functionality is provided by the User::Transferable concern.1  
  3. The user opens this URL or scans the QR code on a second, unauthenticated device.  
  4. The request is handled by the Sessions::TransfersController. The system validates the transfer ID from the URL against the User record.1  
  5. Upon successful validation, the system creates a new session for that user on the second device, logging them in immediately without requiring a password. The user is then redirected to the application interior.

### **Section 5: The Core Chat Experience Journey**

This section details the end-to-end workflow of using the chat application, from viewing rooms to sending and interacting with messages in various rich formats.

#### **5.1. Viewing a Room**

* **Journey:**  
  1. A user selects a room from the sidebar list.  
  2. The application navigates to the room's URL, handled by RoomsController\#show.1 The system loads the last page of messages for that room.  
  3. The client-side messages\_controller.js takes over rendering. It must apply appropriate CSS classes to messages for threading (grouping consecutive messages by the same user within a time window), for messages sent by the current user (the .message--me class, which aligns them to the right), and for displaying date separators between messages posted on different days.1  
  4. The client must establish a real-time connection to the server for the specific room. This involves subscribing to both an ActionCable channel (RoomChannel) for presence and typing notifications, and a Turbo Stream (turbo\_stream\_from @room, :messages) to receive new and updated messages.1  
  5. The act of entering the room must immediately mark it as "read" for the user. This is achieved by the PresenceChannel broadcasting a read event, which the client-side rooms\_list\_controller.js uses to remove any visual "unread" indicators from that room in the sidebar.1

#### **5.2. Composing and Sending a Message**

The message composer is a rich-media input that relies heavily on client-side logic for a seamless user experience.

* **Journey:**  
  1. The user interacts with the Trix rich text editor at the bottom of the room view. The composer\_controller.js manages all composer-related interactions.1  
  2. **Text Formatting:** The user can toggle a rich text toolbar to apply formatting such as bold, italics, blockquotes, and hyperlinks.  
  3. **File Attachments:** The user can attach files via drag-and-drop onto the application window, pasting from the clipboard, or using a traditional file picker. The composer\_controller.js must intercept these events, generate client-side previews for each file (image thumbnails or generic file icons), and display them within the composer area.1  
  4. **Sending:** The user can send the message by pressing Enter (if the rich text toolbar is not active) or Ctrl/Cmd+Enter, or by clicking the send button.  
  5. **Optimistic UI:** Upon submission, the composer\_controller.js must immediately perform an optimistic update. It generates a temporary client\_message\_id (a UUID), creates a "pending" version of the message in the UI, and then submits the form data to the server.1 This provides instant feedback to the user. For file attachments, a pending UI element showing the filename and upload progress must be displayed.  
  6. **Server Processing:** The MessagesController\#create action receives the submission, including the client\_message\_id. It saves the message and any attachments to the database.1  
  7. **Real-Time Broadcast:** The server then broadcasts the final, rendered message back to all clients in the room via Turbo Streams. The broadcasted HTML includes the same client\_message\_id, allowing the client-side messages\_controller.js to seamlessly replace the pending UI element with the confirmed message.1 If the submission fails, the server response must indicate this, and the client must update the pending message to a "failed" state.

#### **5.3. Interacting with Messages**

Once messages are posted, users can interact with them in several ways.

* **@Mentions:** A user types @ followed by another user's name. The rich\_autocomplete\_controller.js must detect this pattern and send an asynchronous request to the Autocompletable::UsersController.1 This endpoint returns a JSON list of users who are members of the current room. When the user selects a name from the suggestion list, the client must insert a special  
  \<action-text-attachment\> tag into the Trix editor, containing the mentioned user's Signed Global ID (SGID).1 This creates a rich mention that is more than just text.  
* **Boosting (Reactions):** A user can react to a message by adding a "boost." The UI must provide an interface to add a boost (e.g., an emoji) to any message. This action is handled by the Messages::BoostsController.1 The creation or deletion of a boost must be broadcast to the room in real-time via Turbo Streams, appending or removing the boost from the corresponding message on all clients' screens.1  
* **Link Unfurling:** When a user pastes a URL into the composer, the client-side Unfurler.js must detect it and send the URL to the UnfurlLinksController.1 The server-side controller fetches the URL, extracts its OpenGraph metadata (title, description, image), and returns it to the client. The client then inserts a rich preview of the link into the message as an  
  \<action-text-attachment\>. The system must handle special cases, such as using a service like fxtwitter.com to fetch metadata for Twitter/X links, as those platforms may not provide standard OpenGraph tags.1  
* **Editing and Deleting:** The message creator or an administrator can edit or delete a message. The UI must provide controls for these actions within a message's options menu. The MessagesController handles these requests, and the changes are broadcast using replace or remove Turbo Stream actions to update all clients in real-time.1  
* **/play Commands:** The system supports slash commands for playing sounds. If a user's message consists solely of /play \<sound\_name\> (e.g., /play trombone), the system must not render it as plain text. Instead, the Message model and messages\_helper.rb must identify this pattern and render a special UI element that allows users to play the corresponding sound file.1

### **Section 6: Room and Membership Management Journey**

This section covers the workflows for creating and managing chat rooms and a user's personal involvement in them.

#### **6.1. Creating Rooms**

The process for creating a room varies significantly based on its type.

* **Journey (Open/Closed Rooms):**  
  1. A user with sufficient permissions (typically an administrator) initiates the "New Room" workflow.  
  2. The UI must present a choice between creating an "Open" or "Closed" room.  
  3. If "Closed" is selected, the UI must display a list of all active users in the account, allowing the creator to select the initial members.1  
  4. The user provides a name for the room.  
  5. The request is processed by the appropriate controller (Rooms::OpensController or Rooms::ClosedsController). The system creates the Room record and its initial Membership records.1  
  6. The new room must be broadcast via Turbo Streams to the sidebars of all users who are members of the new room.  
* **Journey (Direct Message Room):**  
  1. A user initiates a new direct message, typically by selecting one or more users from a list or profile.  
  2. The Rooms::DirectsController\#create action receives the list of user IDs.1  
  3. The system must first check if a Direct room for this exact set of users already exists. If it does, the user is redirected to the existing room.  
  4. If no such room exists, a new Rooms::Direct is created, and all specified users are granted membership. The new room is then broadcast to the sidebars of all its members.1

#### **6.2. Managing Room Settings (Admin/Creator)**

* **Journey:**  
  1. An administrator or the room's creator navigates to the settings page for a specific room.  
  2. The UI must allow them to change the room's name.  
  3. For Closed rooms, the UI must provide an interface to add new members from a list of account users and to remove existing members.  
  4. When changes are submitted, the system updates the Room and revises its Membership list. All changes, such as a name update or a user being added/removed, must be broadcast via Turbo Streams to ensure all affected users' sidebars are updated in real-time.1

#### **6.3. Managing Personal Room Involvement (Any User)**

* **Journey:**  
  1. While viewing a room, a user clicks the notification bell icon located in the room's navigation header.  
  2. This action is handled by the Rooms::InvolvementsController.1  
  3. Each click must cycle the user's involvement status for that specific room through the predefined sequence: mentions → everything → nothing → invisible → mentions.  
  4. The UI must visually reflect the new status (e.g., by changing the bell icon).  
  5. If the status is changed to invisible, a Turbo Stream must be broadcast to that user to remove the room from their sidebar. Conversely, if the status changes from invisible to a visible state, a stream must add it back.1

## **Part III: Specialized User Journeys**

This part details the workflows unique to administrators and bots, which operate under different permission sets and interaction models compared to standard members.

### **Section 7: The Administrator Journey**

Administrators have access to a dedicated set of views for managing the entire Campfire instance.

#### **7.1. Managing Account Settings**

* **Journey:**  
  1. An administrator navigates to the central "Account Settings" page, rendered by AccountsController\#edit.1  
  2. From this interface, the administrator can perform several high-level actions:  
     * **Update Name and Logo:** Modify the account's public name and upload a new logo image.1  
     * **Apply Custom CSS:** Access a separate view via Accounts::CustomStylesController to input and save custom CSS styles that will be applied globally across the application.1  
     * **Regenerate Join Code:** Invalidate the current join\_code and generate a new one to control future user registrations.1

#### **7.2. Managing Users**

* **Journey:**  
  1. The Account Settings page must display a paginated list of all active, non-bot users in the system.  
  2. For each user in the list, the administrator must have controls to perform two key actions, handled by Accounts::UsersController 1:  
     * **Change Role:** Toggle a user's role between Member and Administrator. The change must be submitted via an asynchronous request and take effect immediately.  
     * **Deactivate User:** Remove a user from the account. This action, handled by destroy, must trigger the User\#deactivate method, which performs a "soft delete" by marking the user as inactive, revoking their memberships, deleting their sessions, and anonymizing their email address.1

#### **7.3. Managing Bots**

* **Journey:**  
  1. An administrator navigates to a dedicated "Bots" management page, rendered by Accounts::BotsController.1  
  2. The page lists all active bots. From here, an administrator can:  
     * **Create a New Bot:** Provide a name, an avatar, and an optional webhook URL to register a new bot user.  
     * **Edit a Bot:** Modify an existing bot's name, avatar, or webhook URL.  
     * **Reset API Key:** For any bot, the administrator can reset its bot\_token (API key) via the Accounts::Bots::KeysController. This invalidates the old key and generates a new one, which must be displayed to the administrator for configuration in the external service.1  
     * **Deactivate a Bot:** Remove a bot from the system using the same deactivation process as for regular users.

### **Section 8: The Bot Interaction Journey**

Bots interact with Campfire programmatically through a well-defined webhook and API flow.

#### **8.1. Receiving a Message (Webhook Trigger)**

* **Trigger:** A bot is triggered when a user @mentions it in any room, or when any message is posted in a Direct Message room of which the bot is a member.  
* **Journey:**  
  1. When a message is created, the MessagesController\#create action checks if any bots are eligible to receive a webhook for it.1  
  2. If so, a Bot::WebhookJob is enqueued to handle the outbound communication asynchronously.  
  3. The background job executes the Webhook\#deliver method.1  
  4. The system constructs a detailed JSON payload containing information about the original message, its sender, and the room. This payload must conform to the specification in Section 8.3.  
  5. The system sends this payload as an HTTP POST request to the bot's configured webhook\_url, with a timeout of 7 seconds.1

#### **8.2. Responding to a Message (Bot API)**

* **Journey:**  
  1. The external bot service receives the webhook payload and processes it.  
  2. To send a response back to the Campfire room, the service must make an authenticated HTTP POST request to a specific API endpoint: /rooms/:room\_id/bot/messages.  
  3. Authentication must be performed by including the bot's unique bot\_key as a URL query parameter (e.g., ?bot\_key=...).  
  4. The request is handled by the Messages::ByBotsController, which is specifically configured to allow\_bot\_access.1 The controller authenticates the bot via its key.  
  5. The bot can respond in two ways:  
     * **Text Response:** The raw request body is treated as the message content.  
     * **Attachment Response:** The request is sent as a multipart form upload with a file attached.  
  6. The controller creates a new message in the specified room with the bot as the creator and broadcasts it to all room members via Turbo Streams.

#### **8.3. Webhook Payload and Response Specification**

The following table defines the API contract for bot interactions.

| Direction | Type | Specification |
| :---- | :---- | :---- |
| **Outgoing** | **JSON Payload** | Sent via HTTP POST to the bot's configured webhook\_url. The payload has the following structure: { "user": { "id": number, "name": string }, "room": { "id": number, "name": string, "path": string }, "message": { "id": number, "body": { "html": string, "plain": string }, "path": string } } \- message.body.plain excludes the mention of the recipient bot. |
| **Incoming** | **API Endpoint** | POST /rooms/:room\_id/bot/messages?bot\_key=\<BOT\_KEY\> |
|  | **Text Response** | The raw HTTP request body is used as the message content. Content-Type should be text/plain or text/html. |
|  | **Attachment Response** | The request must be a multipart/form-data POST, with the file included in an attachment parameter. |

## **Part IV: Platform and Non-Functional Requirements**

This final part details system-wide requirements related to deployment, platform-specific features, and the real-time architecture that underpins the user experience.

### **Section 9: Platform-Specific Functionality**

#### **9.1. Progressive Web App (PWA) Support**

The application must function as an installable Progressive Web App to provide a native-like experience on supported devices.

* **Manifest File:** The system must serve a manifest.json file from a stable root URL. This file, handled by the PwaController, must contain the necessary metadata for PWA installation, including the application's name, icons, and start URL. The manifest must be rendered via ERB to allow for dynamic asset path generation.1  
* **Service Worker:** The system must serve a service-worker.js file, also from a stable root URL via the PwaController. This service worker is responsible for enabling offline functionality and managing the client-side logic for web push notifications.1 Both the manifest and service worker endpoints must be publicly accessible without authentication.

#### **9.2. Deployment and Configuration**

The application is designed for containerized deployment and must adhere to the following specifications.

* **Containerization:** The application must be packaged as a Docker container using the provided Dockerfile. This image must be self-contained, including the Ruby on Rails application, the Resque background job processor, a Redis server for caching and ActionCable, and the Thruster web server for SSL termination.1  
* **Configuration:** All critical runtime configurations must be manageable via environment variables, as specified in the README.md. This includes, but is not limited to:  
  * SSL\_DOMAIN: For automatic SSL certificate provisioning via Let's Encrypt.  
  * VAPID\_PUBLIC\_KEY & VAPID\_PRIVATE\_KEY: For enabling web push notifications.  
  * SENTRY\_DSN: For configuring production error reporting to Sentry.  
  * SECRET\_KEY\_BASE: For Rails' cryptographic operations.

### **Section 10: Real-Time System Requirements**

The core user experience of Campfire depends on a robust real-time architecture.

* **User Presence Tracking:** The system must track which users are actively connected to each room on a granular level. The PresenceChannel is responsible for handling user subscriptions to a room's channel. Upon subscription, a user's Membership record must be marked as present, updating its connected\_at timestamp. A connection is considered active if this timestamp is within the last 60 seconds.1 This status is used to determine whether to send push notifications or mark rooms as unread.  
* **Unread Message Indicators:** The system must provide immediate visual feedback when a room contains unread messages. When a message is created, the server must broadcast an event on the UnreadRoomsChannel containing the roomId.1 Client-side JavaScript (  
  rooms\_list\_controller.js) listening to this channel must check if the user is currently in that room. If not, it must apply an .unread CSS class to the room's entry in the sidebar. When a user enters a room, the PresenceChannel or ReadRoomsChannel must trigger a client-side event that removes this class.1  
* **Typing Notifications:** To enhance the conversational flow, the system must display a "user is typing..." indicator. The client-side typing\_notifications\_controller.js must send start and stop events to the TypingNotificationsChannel as a user types and pauses in the composer. The server then broadcasts these events to all other members of the room, allowing their clients to display and hide the typing indicator accordingly.1

### **Conclusion**

The Campfire application is a sophisticated, real-time chat system built on a single-tenant architecture. Its design prioritizes a seamless and interactive user experience through heavy reliance on client-side logic, optimistic UI updates, and real-time communication channels like Turbo Streams and ActionCable.

Key architectural patterns that must be preserved in any reimplementation include:

* **The clear separation of user roles**, particularly the distinct, API-driven nature of Bot users.  
* **The granular, per-room presence tracking** managed through the Membership model, which is the cornerstone of the notification and unread status system.  
* **The use of client-side generated IDs** for optimistic UI updates, which is critical for maintaining a responsive feel during message and file submission.  
* **The Single Table Inheritance model for Rooms**, which provides a clean and extensible way to manage different types of conversation spaces with unique business logic.

By adhering to the detailed journeys and requirements specified in this document, a faithful and robust reconstruction of the Campfire application can be achieved, capturing the nuances of its user-centric and real-time design.

#### **Works cited**

1. basecamp-once-campfire-8a5edab282632443.txt