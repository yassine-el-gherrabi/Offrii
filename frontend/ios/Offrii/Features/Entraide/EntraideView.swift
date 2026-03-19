import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = EntraideViewModel()
    @State private var selectedSegment = 0
    @State private var showCreateSheet = false
    @State private var selectedWishId: UUID?
    @State private var messagesWishId: UUID?
    @State private var reportWishId: UUID?

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            VStack(spacing: 0) {
                SectionHeader(
                    title: NSLocalizedString("entraide.title", comment: ""),
                    subtitle: NSLocalizedString("entraide.subtitle", comment: ""),
                    variant: .entraide
                ) {
                    NavigationLink(destination: ProfileView()) {
                        ProfileAvatarButton(
                            initials: ProfileAvatarButton.initials(
                                from: authManager.currentUser?.displayName
                            ),
                            avatarUrl: authManager.currentUser?.avatarUrl
                                .flatMap { URL(string: $0) }
                        )
                    }
                }

                Picker("", selection: $selectedSegment) {
                    Text(NSLocalizedString("entraide.segment.discover", comment: "")).tag(0)
                    Text(NSLocalizedString("entraide.segment.myNeeds", comment: "")).tag(1)
                    Text(NSLocalizedString("entraide.segment.myOffers", comment: "")).tag(2)
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)

                switch selectedSegment {
                case 0:
                    EntraideDiscoverContent(
                        viewModel: viewModel,
                        selectedWishId: $selectedWishId
                    )
                case 1:
                    EntraideMyNeedsContent(
                        selectedWishId: $selectedWishId,
                        showCreateSheet: $showCreateSheet
                    )
                case 2:
                    EntraideMyOffersContent(
                        viewModel: viewModel,
                        selectedWishId: $selectedWishId
                    )
                default:
                    EmptyView()
                }
            }

            // FAB — only on Discover
            if selectedSegment == 0 {
                OffriiFloatingActionButton(
                    icon: "plus",
                    label: NSLocalizedString("entraide.fab.publish", comment: "")
                ) {
                    showCreateSheet = true
                }
                .padding(.trailing, OffriiTheme.spacingLG)
                .padding(.bottom, OffriiTheme.spacingLG)
            }
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .sheet(isPresented: $showCreateSheet, onDismiss: {
            Task { await viewModel.loadWishes() }
        }) {
            CreateWishSheet()
                .presentationDetents([.large])
        }
        .sheet(item: $selectedWishId, onDismiss: {
            Task { await viewModel.loadWishes() }
        }) { wishId in
            WishDetailSheet(
                wishId: wishId,
                onOpenMessages: { messagesWishId = wishId },
                onReport: { reportWishId = wishId }
            )
            .environment(authManager)
            .presentationDetents([.medium, .large])
        }
        .sheet(item: $messagesWishId) { wishId in
            WishMessagesSheet(wishId: wishId)
                .presentationDetents([.large])
        }
        .sheet(item: $reportWishId) { wishId in
            ReportWishSheet(wishId: wishId)
                .presentationDetents([.medium])
        }
        .task {
            await viewModel.loadWishes()
        }
    }
}
