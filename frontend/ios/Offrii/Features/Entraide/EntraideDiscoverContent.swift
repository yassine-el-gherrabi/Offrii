import SwiftUI

// MARK: - Discover Content

struct EntraideDiscoverContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0, pinnedViews: .sectionHeaders) {
                Section {
                    if viewModel.isLoading && viewModel.wishes.isEmpty {
                        skeletonGrid
                    } else if viewModel.wishes.isEmpty {
                        Spacer().frame(height: 60)
                        OffriiEmptyState(
                            icon: "heart.circle",
                            title: NSLocalizedString("entraide.empty", comment: ""),
                            subtitle: NSLocalizedString("entraide.emptySubtitle", comment: "")
                        )
                        Spacer()
                    } else {
                        wishGrid
                    }
                } header: {
                    categoryChips
                        .background(OffriiTheme.background)
                }
            }
        }
        .refreshable {
            await viewModel.loadWishes()
        }
    }

    // MARK: - Category Chips

    private var categoryChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                let allSelected = viewModel.selectedCategory == nil

                Button {
                    Task { await viewModel.selectCategory(nil) }
                } label: {
                    chipLabel(
                        icon: "sparkles",
                        text: NSLocalizedString("entraide.category.all", comment: ""),
                        isSelected: allSelected
                    )
                }
                .buttonStyle(.plain)

                ForEach(WishCategory.allCases) { category in
                    let isSelected = viewModel.selectedCategory == category
                    Button {
                        Task { await viewModel.selectCategory(category) }
                    } label: {
                        chipLabel(
                            icon: categoryIcon(category),
                            text: category.label,
                            isSelected: isSelected
                        )
                    }
                    .buttonStyle(.plain)
                    .animation(OffriiAnimation.snappy, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingXS)
        }
    }

    private func chipLabel(icon: String, text: String, isSelected: Bool) -> some View {
        HStack(spacing: 4) {
            Image(systemName: icon)
                .font(.system(size: 11))
            Text(text)
                .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
        }
        .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
        .padding(.horizontal, OffriiTheme.spacingMD)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(isSelected ? OffriiTheme.primary : .white)
        .cornerRadius(OffriiTheme.cornerRadiusXL)
        .overlay(
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
        )
    }

    private func categoryIcon(_ category: WishCategory) -> String {
        switch category {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "tag.fill"
        }
    }

    // MARK: - Wish Grid

    private var wishGrid: some View {
        LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
            ForEach(viewModel.filteredWishes) { wish in
                EntraideWishCard(wish: wish) {
                    selectedWishId = wish.id
                }
                .onAppear {
                    Task { await viewModel.loadMoreIfNeeded(currentWish: wish) }
                }
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
    }

    // MARK: - Skeleton

    private var skeletonGrid: some View {
        LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
            ForEach(0..<6, id: \.self) { _ in
                SkeletonGridCard()
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
    }
}
