import SwiftUI

struct AddFriendSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var searchText = ""
    @State private var results: [UserSearchResult] = []
    @State private var isSearching = false
    @State private var sentUsernames: Set<String> = []
    @State private var error: String?
    let onSent: () -> Void

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.cardSurface.ignoresSafeArea()

                VStack(spacing: 0) {
                    // Search field
                    HStack(spacing: OffriiTheme.spacingSM) {
                        Image(systemName: "magnifyingglass")
                            .foregroundColor(OffriiTheme.textMuted)
                        TextField(
                            NSLocalizedString("friends.add.searchPlaceholder", comment: ""),
                            text: $searchText
                        )
                        .font(OffriiTypography.body)
                        .autocapitalization(.none)
                        .autocorrectionDisabled()

                        if !searchText.isEmpty {
                            Button {
                                searchText = ""
                                results = []
                            } label: {
                                Image(systemName: "xmark.circle.fill")
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                        }
                    }
                    .padding(OffriiTheme.spacingMD)
                    .background(OffriiTheme.card)
                    .cornerRadius(OffriiTheme.cornerRadiusSM)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                            .stroke(OffriiTheme.border, lineWidth: 1)
                    )
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.top, OffriiTheme.spacingSM)

                    if let error {
                        Text(error)
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.danger)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingSM)
                    }

                    if isSearching {
                        Spacer()
                        ProgressView()
                        Spacer()
                    } else if results.isEmpty && !searchText.isEmpty {
                        Spacer()
                        Text(NSLocalizedString("friends.add.noResults", comment: ""))
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.textSecondary)
                        Spacer()
                    } else {
                        List {
                            ForEach(results) { user in
                                HStack(spacing: OffriiTheme.spacingSM) {
                                    AvatarView(user.displayName ?? user.username, size: .small)

                                    VStack(alignment: .leading, spacing: 2) {
                                        if let name = user.displayName {
                                            Text(name)
                                                .font(OffriiTypography.body)
                                                .foregroundColor(OffriiTheme.text)
                                        }
                                        Text("@\(user.username)")
                                            .font(OffriiTypography.caption)
                                            .foregroundColor(OffriiTheme.textMuted)
                                    }

                                    Spacer()

                                    if sentUsernames.contains(user.username) {
                                        Text(NSLocalizedString("friends.add.sent", comment: ""))
                                            .font(OffriiTypography.caption)
                                            .foregroundColor(OffriiTheme.success)
                                    } else {
                                        Button {
                                            Task { await sendRequest(username: user.username) }
                                        } label: {
                                            Text(NSLocalizedString("friends.add.send", comment: ""))
                                                .font(OffriiTypography.footnote)
                                                .fontWeight(.semibold)
                                                .foregroundColor(.white)
                                                .padding(.horizontal, OffriiTheme.spacingSM)
                                                .padding(.vertical, OffriiTheme.spacingXS)
                                                .background(OffriiTheme.primary)
                                                .cornerRadius(OffriiTheme.cornerRadiusXL)
                                        }
                                    }
                                }
                            }
                        }
                        .listStyle(.plain)
                    }
                }
            }
            .navigationTitle(NSLocalizedString("friends.add.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                }
            }
            .onChange(of: searchText) { _, newValue in
                Task { await search(query: newValue) }
            }
        }
    }

    private func search(query: String) async {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        guard trimmed.count >= 2 else {
            results = []
            return
        }

        isSearching = true
        do {
            results = try await FriendService.shared.searchUsers(query: trimmed)
        } catch {
            self.error = error.localizedDescription
        }
        isSearching = false
    }

    private func sendRequest(username: String) async {
        error = nil
        do {
            _ = try await FriendService.shared.sendRequest(username: username)
            sentUsernames.insert(username)
            onSent()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
