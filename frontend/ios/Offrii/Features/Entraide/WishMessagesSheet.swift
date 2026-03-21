import SwiftUI

// MARK: - Wish Messages Sheet

struct WishMessagesSheet: View {
    let wishId: UUID

    @Environment(\.dismiss) private var dismiss
    @State private var messages: [WishMessage] = []
    @State private var messageText = ""
    @State private var isLoading = false
    @State private var isSending = false
    @State private var sendCooldown = false
    @State private var currentPage = 1
    @State private var hasMorePages = false
    @State private var pollingTask: Task<Void, Never>?
    @State private var lastActivityTime = Date()

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                if isLoading && messages.isEmpty {
                    VStack(spacing: OffriiTheme.spacingSM) {
                        ForEach(0..<5, id: \.self) { _ in
                            SkeletonRow(height: 56)
                        }
                    }
                    .padding(.top, OffriiTheme.spacingBase)
                    Spacer()
                } else if messages.isEmpty {
                    Spacer()
                    OffriiEmptyState(
                        icon: "bubble.left.and.bubble.right",
                        title: NSLocalizedString("entraide.messages.empty", comment: ""),
                        subtitle: NSLocalizedString("entraide.messages.emptySubtitle", comment: "")
                    )
                    Spacer()
                } else {
                    messageList
                }

                inputBar
            }
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("entraide.action.messages", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.ok", comment: "")) { dismiss() }
                }
            }
        }
        .task {
            await loadMessages()
            startPolling()
        }
        .onDisappear {
            pollingTask?.cancel()
        }
    }

    // MARK: - Message List

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingSM) {
                    if hasMorePages {
                        Button {
                            Task { await loadOlderMessages() }
                        } label: {
                            Text(NSLocalizedString("entraide.messages.loadMore", comment: ""))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.primary)
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, OffriiTheme.spacingSM)
                        }
                    }

                    ForEach(messages) { message in
                        MessageBubble(
                            text: message.body,
                            senderName: message.senderDisplayName,
                            timestamp: message.createdAt,
                            isMine: message.isMine
                        )
                        .id(message.id)
                    }
                }
                .padding(OffriiTheme.spacingBase)
            }
            .onAppear {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                    if let lastId = messages.last?.id {
                        proxy.scrollTo(lastId, anchor: .bottom)
                    }
                }
            }
            .onChange(of: messages.count) { _, _ in
                if let lastId = messages.last?.id {
                    withAnimation {
                        proxy.scrollTo(lastId, anchor: .bottom)
                    }
                }
            }
        }
    }

    // MARK: - Input Bar

    private let messageMaxLength = 500

    private var isMessageOverLimit: Bool {
        messageText.count > messageMaxLength
    }

    private var inputBar: some View {
        VStack(spacing: 0) {
            HStack(spacing: OffriiTheme.spacingSM) {
                LimitedTextEditor(
                    placeholder: NSLocalizedString("entraide.messages.placeholder", comment: ""),
                    text: $messageText,
                    maxLength: messageMaxLength,
                    lineLimit: 1...4
                )

                Button {
                    Task { await sendMessage() }
                } label: {
                    Image(systemName: "arrow.up.circle.fill")
                        .font(.system(size: 32))
                        .foregroundColor(
                            messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                                || isMessageOverLimit || sendCooldown
                                ? OffriiTheme.textMuted
                                : OffriiTheme.primary
                        )
                }
                .disabled(
                    messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                        || isMessageOverLimit || isSending || sendCooldown
                )
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(OffriiTheme.card)
    }

    // MARK: - Actions

    private func loadMessages() async {
        isLoading = true
        currentPage = 1
        do {
            let response = try await WishMessageService.shared.listMessages(
                wishId: wishId, page: 1, limit: 50
            )
            messages = response.data
            hasMorePages = response.pagination.hasMore
        } catch {}
        isLoading = false
    }

    private func loadOlderMessages() async {
        let nextPage = currentPage + 1
        do {
            let response = try await WishMessageService.shared.listMessages(
                wishId: wishId, page: nextPage, limit: 50
            )
            messages.insert(contentsOf: response.data, at: 0)
            hasMorePages = response.pagination.hasMore
            currentPage = nextPage
        } catch {}
    }

    private func sendMessage() async {
        let text = messageText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }
        isSending = true

        do {
            let msg = try await WishMessageService.shared.sendMessage(wishId: wishId, body: text)
            messages.append(msg)
            messageText = ""
            lastActivityTime = Date()
            OffriiHaptics.tap()

            // 2s cooldown to prevent spam
            sendCooldown = true
            try? await Task.sleep(for: .seconds(2))
            sendCooldown = false
        } catch {}
        isSending = false
    }

    private func startPolling() {
        pollingTask = Task {
            while !Task.isCancelled {
                let elapsed = Date().timeIntervalSince(lastActivityTime)
                let interval: Double
                if elapsed < 30 {
                    interval = 3
                } else if elapsed < 120 {
                    interval = 10
                } else {
                    interval = 30
                }

                try? await Task.sleep(for: .seconds(interval))
                guard !Task.isCancelled else { break }
                await refreshMessages()
            }
        }
    }

    private func refreshMessages() async {
        guard let response = try? await WishMessageService.shared.listMessages(
            wishId: wishId, page: 1, limit: 100
        ) else { return }

        if response.data.last?.id != messages.last?.id {
            messages = response.data
            lastActivityTime = Date()
        }
    }
}
