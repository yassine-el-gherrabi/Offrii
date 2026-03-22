import SwiftUI

struct EditShareLinkSheet: View {
    let link: ShareLinkResponse
    let onUpdated: (ShareLinkResponse) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var label: String
    @State private var permission: String
    @State private var isActive: Bool
    @State private var linkTTL: LinkTTL
    @State private var isSaving = false

    init(link: ShareLinkResponse, onUpdated: @escaping (ShareLinkResponse) -> Void) {
        self.link = link
        self.onUpdated = onUpdated
        _label = State(initialValue: link.label ?? "")
        _permission = State(initialValue: link.permissions ?? "view_and_claim")
        _isActive = State(initialValue: link.isActive ?? true)
        _linkTTL = State(initialValue: link.expiresAt != nil ? .oneWeek : .never)
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
                    // Label
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text(NSLocalizedString("share.linkName", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.textMuted)

                        TextField(NSLocalizedString("share.linkNamePlaceholder", comment: ""), text: $label)
                            .font(OffriiTypography.body)
                            .padding(.horizontal, OffriiTheme.spacingMD)
                            .padding(.vertical, 10)
                            .background(OffriiTheme.surface)
                            .cornerRadius(OffriiTheme.cornerRadiusSM)
                            .overlay(
                                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                                    .strokeBorder(OffriiTheme.border, lineWidth: 1)
                            )
                    }

                    // Permissions
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text(NSLocalizedString("share.permissions", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.textMuted)

                        Picker("", selection: $permission) {
                            Text(NSLocalizedString("share.permViewAndClaim", comment: "")).tag("view_and_claim")
                            Text(NSLocalizedString("share.permViewOnly", comment: "")).tag("view_only")
                        }
                        .pickerStyle(.segmented)
                    }

                    // Expiration
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text(NSLocalizedString("share.ttl", comment: ""))
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.textMuted)

                        Picker("", selection: $linkTTL) {
                            ForEach(LinkTTL.allCases, id: \.rawValue) { ttl in
                                Text(ttl.label).tag(ttl)
                            }
                        }
                        .pickerStyle(.segmented)
                    }

                    // Active toggle
                    Toggle(isOn: $isActive) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(NSLocalizedString("share.linkActive", comment: ""))
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.text)
                            Text(NSLocalizedString("share.linkActiveHint", comment: ""))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                    }
                    .tint(OffriiTheme.primary)

                    // Save
                    OffriiButton(
                        NSLocalizedString("common.save", comment: ""),
                        isLoading: isSaving
                    ) {
                        Task { await save() }
                    }
                }
                .padding(OffriiTheme.spacingLG)
            }
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("wishlist.edit", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) { dismiss() }
                        .foregroundColor(OffriiTheme.primary)
                }
            }
        }
    }

    private func save() async {
        isSaving = true
        let trimmedLabel = label.trimmingCharacters(in: .whitespaces)

        let body = UpdateShareLinkBody(
            label: trimmedLabel.isEmpty ? nil : trimmedLabel,
            permissions: permission,
            isActive: isActive,
            expiresAt: linkTTL.expiresAt
        )

        do {
            let _: ShareLinkResponse = try await APIClient.shared.request(.updateShareLink(id: link.id, body: body))
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime]
            let newExpiry = linkTTL.expiresAt.flatMap { formatter.date(from: $0) }

            onUpdated(ShareLinkResponse(
                id: link.id,
                token: link.token,
                url: link.url,
                label: trimmedLabel.isEmpty ? nil : trimmedLabel,
                permissions: permission,
                scope: link.scope,
                isActive: isActive,
                createdAt: link.createdAt,
                expiresAt: newExpiry
            ))
            OffriiHaptics.success()
            dismiss()
        } catch {
            OffriiHaptics.error()
        }
        isSaving = false
    }
}
