import SwiftUI

// MARK: - WishMessagesView

struct WishMessagesView: View {
    let wishId: UUID
    let wishTitle: String
    @State private var viewModel = WishMessagesViewModel()

    var body: some View {
        VStack(spacing: 0) {
            // Messages list
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: OffriiTheme.spacingSM) {
                        ForEach(viewModel.messages) { message in
                            MessageBubble(
                                text: message.body,
                                senderName: message.isMine ? nil : message.senderDisplayName,
                                timestamp: message.createdAt,
                                isMine: message.isMine
                            )
                            .id(message.id)
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                }
                .onChange(of: viewModel.messages.count) { _, _ in
                    withAnimation {
                        if let lastId = viewModel.messages.last?.id {
                            proxy.scrollTo(lastId, anchor: .bottom)
                        }
                    }
                }
                .onAppear {
                    if let lastId = viewModel.messages.last?.id {
                        proxy.scrollTo(lastId, anchor: .bottom)
                    }
                }
            }

            Divider()

            // Input bar
            inputBar
        }
        .background(OffriiTheme.cardSurface)
        .navigationTitle(wishTitle)
        .navigationBarTitleDisplayMode(.inline)
        .task {
            await viewModel.loadMessages(wishId: wishId)
            viewModel.startPolling(wishId: wishId)
        }
        .onDisappear {
            viewModel.stopPolling()
        }
    }

    // MARK: - Input Bar

    private var inputBar: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            TextField(
                NSLocalizedString("entraide.messages.placeholder", comment: ""),
                text: $viewModel.messageText
            )
            .font(OffriiTypography.body)
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusXL)

            Button {
                Task { await viewModel.sendMessage(wishId: wishId) }
            } label: {
                Image(systemName: "paperplane.fill")
                    .font(.system(size: 18))
                    .foregroundColor(.white)
                    .frame(width: 36, height: 36)
                    .background(
                        viewModel.messageText.trimmingCharacters(in: .whitespaces).isEmpty
                            ? OffriiTheme.textMuted
                            : OffriiTheme.primary
                    )
                    .cornerRadius(18)
            }
            .disabled(
                viewModel.messageText.trimmingCharacters(in: .whitespaces).isEmpty
                    || viewModel.isSending
            )
        }
        .padding(.horizontal, OffriiTheme.spacingMD)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(OffriiTheme.card)
    }
}
