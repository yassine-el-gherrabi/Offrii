import Foundation
import Combine

@Observable
@MainActor
final class WishMessagesViewModel {
    var messages: [WishMessage] = []
    var isLoading = false
    var isSending = false
    var error: String?
    var messageText = ""

    private var timerCancellable: AnyCancellable?

    // MARK: - Load Messages

    func loadMessages(wishId: UUID) async {
        isLoading = true
        error = nil

        do {
            // Load all messages (large limit for chat view)
            let response = try await WishMessageService.shared.listMessages(
                wishId: wishId,
                page: 1,
                limit: 100
            )
            messages = response.data
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    // MARK: - Send Message

    func sendMessage(wishId: UUID) async {
        let text = messageText.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }

        isSending = true
        do {
            let message = try await WishMessageService.shared.sendMessage(
                wishId: wishId,
                body: text
            )
            messages.append(message)
            messageText = ""
        } catch {
            self.error = error.localizedDescription
        }
        isSending = false
    }

    // MARK: - Polling

    func startPolling(wishId: UUID) {
        timerCancellable = Timer.publish(every: 10, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                guard let self else { return }
                Task { @MainActor in
                    await self.refreshMessages(wishId: wishId)
                }
            }
    }

    func stopPolling() {
        timerCancellable?.cancel()
        timerCancellable = nil
    }

    private func refreshMessages(wishId: UUID) async {
        do {
            let response = try await WishMessageService.shared.listMessages(
                wishId: wishId,
                page: 1,
                limit: 100
            )
            messages = response.data
        } catch {
            // Silent refresh failure
        }
    }
}
