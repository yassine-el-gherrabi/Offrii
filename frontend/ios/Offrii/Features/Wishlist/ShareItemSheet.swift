import SwiftUI

/// Creates a share link for a single item and presents sharing options.
struct ShareItemSheet: View {
    let itemId: UUID
    @Environment(\.dismiss) private var dismiss
    @State private var isCreating = true
    @State private var shareUrl: URL?
    @State private var error: String?

    var body: some View {
        NavigationStack {
            VStack(spacing: OffriiTheme.spacingLG) {
                Spacer()

                if isCreating {
                    ProgressView()
                        .scaleEffect(1.2)
                    Text(NSLocalizedString("share.creating", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textMuted)
                } else if let url = shareUrl {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.system(size: 40))
                        .foregroundColor(OffriiTheme.success)

                    Text(NSLocalizedString("share.linkReady", comment: ""))
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.text)

                    // Copy button
                    Button {
                        UIPasteboard.general.string = url.absoluteString
                        OffriiHaptics.success()
                    } label: {
                        HStack(spacing: OffriiTheme.spacingSM) {
                            Image(systemName: "doc.on.doc")
                            Text(NSLocalizedString("share.copyLink", comment: ""))
                        }
                        .font(OffriiTypography.headline)
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, OffriiTheme.spacingMD)
                        .background(OffriiTheme.primary)
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                    }

                    // Share via iOS
                    ShareLink(item: url) {
                        HStack(spacing: OffriiTheme.spacingSM) {
                            Image(systemName: "square.and.arrow.up")
                            Text(NSLocalizedString("share.sendVia", comment: ""))
                        }
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.primary)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, OffriiTheme.spacingMD)
                        .background(OffriiTheme.primary.opacity(0.1))
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                    }
                } else if let error {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 40))
                        .foregroundColor(OffriiTheme.danger)
                    Text(error)
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.danger)
                }

                Spacer()
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("share.item", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) { dismiss() }
                        .foregroundColor(OffriiTheme.primary)
                }
            }
            .task {
                await createShareLink()
            }
        }
    }

    private func createShareLink() async {
        isCreating = true
        do {
            let body = CreateShareLinkBody.shareItem(id: itemId)
            let response: ShareLinkResponse = try await APIClient.shared.request(.createShareLink(body))
            shareUrl = URL(string: response.displayUrl)
            OffriiHaptics.success()
        } catch {
            self.error = error.localizedDescription
            OffriiHaptics.error()
        }
        isCreating = false
    }
}
