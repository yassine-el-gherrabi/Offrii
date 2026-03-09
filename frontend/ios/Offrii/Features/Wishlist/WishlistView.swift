import SwiftUI

struct WishlistView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = WishlistViewModel()
    @State private var showQuickAdd = false
    @State private var showSortMenu = false

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            OffriiTheme.cardSurface.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                HeaderView(
                    title: NSLocalizedString("wishlist.title", comment: ""),
                    subtitle: nil
                )

                // Category chips
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        chipButton(
                            title: NSLocalizedString("wishlist.allCategories", comment: ""),
                            isSelected: viewModel.selectedCategoryId == nil
                        ) {
                            viewModel.selectCategory(nil)
                        }

                        ForEach(viewModel.categories, id: \.id) { category in
                            chipButton(
                                title: category.name,
                                isSelected: viewModel.selectedCategoryId == category.id
                            ) {
                                viewModel.selectCategory(category.id)
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingSM)
                }

                // Segmented + sort
                HStack {
                    Picker("", selection: Binding(
                        get: { viewModel.filteredSegmentIndex },
                        set: { viewModel.changeSegment($0) }
                    )) {
                        Text(NSLocalizedString("wishlist.active", comment: "")).tag(0)
                        Text(NSLocalizedString("wishlist.purchased", comment: "")).tag(1)
                    }
                    .pickerStyle(.segmented)

                    Menu {
                        Button {
                            viewModel.changeSort("created_at")
                        } label: {
                            Label(
                                NSLocalizedString("wishlist.sort.date", comment: ""),
                                systemImage: viewModel.sortField == "created_at" ? "checkmark" : ""
                            )
                        }
                        Button {
                            viewModel.changeSort("priority")
                        } label: {
                            Label(
                                NSLocalizedString("wishlist.sort.priority", comment: ""),
                                systemImage: viewModel.sortField == "priority" ? "checkmark" : ""
                            )
                        }
                        Button {
                            viewModel.changeSort("name")
                        } label: {
                            Label(
                                NSLocalizedString("wishlist.sort.name", comment: ""),
                                systemImage: viewModel.sortField == "name" ? "checkmark" : ""
                            )
                        }
                    } label: {
                        Image(systemName: "arrow.up.arrow.down")
                            .foregroundColor(OffriiTheme.primary)
                            .padding(OffriiTheme.spacingSM)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)

                // List
                if viewModel.isLoading && viewModel.items.isEmpty {
                    Spacer()
                    ProgressView()
                    Spacer()
                } else if viewModel.items.isEmpty {
                    Spacer()
                    VStack(spacing: OffriiTheme.spacingSM) {
                        Image(systemName: "gift")
                            .font(.system(size: 48))
                            .foregroundColor(OffriiTheme.textMuted)
                        Text(NSLocalizedString("wishlist.empty", comment: ""))
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.textSecondary)
                        Text(NSLocalizedString("wishlist.emptySubtitle", comment: ""))
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.textMuted)
                    }
                    Spacer()
                } else {
                    List {
                        ForEach(viewModel.items) { item in
                            NavigationLink(value: item.id) {
                                ItemRow(
                                    item: item,
                                    categoryName: viewModel.categoryName(for: item.categoryId)
                                )
                            }
                            .listRowBackground(OffriiTheme.card)
                            .listRowSeparatorTint(OffriiTheme.border)
                            .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                                Button(role: .destructive) {
                                    Task { await viewModel.deleteItem(item) }
                                } label: {
                                    Label(
                                        NSLocalizedString("common.delete", comment: ""),
                                        systemImage: "trash"
                                    )
                                }
                            }
                            .swipeActions(edge: .leading, allowsFullSwipe: true) {
                                if item.isActive {
                                    Button {
                                        Task { await viewModel.markPurchased(item) }
                                    } label: {
                                        Label(
                                            NSLocalizedString("wishlist.markReceived", comment: ""),
                                            systemImage: "checkmark.circle"
                                        )
                                    }
                                    .tint(OffriiTheme.success)
                                }
                            }
                            .onAppear {
                                Task { await viewModel.loadMoreIfNeeded(currentItem: item) }
                            }
                        }

                        if viewModel.isLoadingMore {
                            HStack {
                                Spacer()
                                ProgressView()
                                Spacer()
                            }
                            .listRowBackground(Color.clear)
                        }
                    }
                    .listStyle(.plain)
                    .scrollContentBackground(.hidden)
                    .navigationDestination(for: UUID.self) { itemId in
                        ItemDetailView(itemId: itemId)
                            .environment(authManager)
                    }
                }
            }

            // FAB
            Button {
                showQuickAdd = true
            } label: {
                Image(systemName: "plus")
                    .font(.system(size: 24, weight: .semibold))
                    .foregroundColor(.white)
                    .frame(width: 56, height: 56)
                    .background(OffriiTheme.primary)
                    .clipShape(Circle())
                    .shadow(color: OffriiTheme.primary.opacity(0.3), radius: 8, y: 4)
            }
            .padding(.trailing, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingLG)
        }
        .refreshable {
            await viewModel.loadItems()
        }
        .sheet(isPresented: $showQuickAdd) {
            QuickAddSheet { name in
                await viewModel.quickAdd(name: name)
            }
        }
        .task {
            await viewModel.loadCategories()
            await viewModel.loadItems()
        }
    }

    private func chipButton(title: String, isSelected: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(title)
                .font(OffriiTypography.subheadline)
                .fontWeight(isSelected ? .semibold : .regular)
                .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                .padding(.horizontal, OffriiTheme.spacingMD)
                .padding(.vertical, OffriiTheme.spacingSM)
                .background(isSelected ? OffriiTheme.primary : OffriiTheme.card)
                .cornerRadius(OffriiTheme.cornerRadiusXL)
                .overlay(
                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                        .strokeBorder(
                            isSelected ? Color.clear : OffriiTheme.border,
                            lineWidth: 1
                        )
                )
        }
    }
}
