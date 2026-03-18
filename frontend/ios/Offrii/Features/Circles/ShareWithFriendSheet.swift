import SwiftUI

// MARK: - Share With Friend Sheet

struct ShareWithFriendSheet: View {
    let circleId: UUID
    let friendName: String

    @Environment(\.dismiss) private var dismiss
    @State private var currentMode = "none"
    @State private var selectedMode = "none"
    @State private var categories: [CategoryResponse] = []
    @State private var selectedCategoryIds: Set<UUID> = []
    @State private var items: [Item] = []
    @State private var selectedItemIds: Set<UUID> = []
    @State private var initialSharedItemIds: Set<UUID> = []
    @State private var isLoading = true
    @State private var isSaving = false
    @State private var privateCount = 0

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if isLoading {
                    SkeletonList(count: 4)
                } else {
                    ScrollView {
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                            modeSelector
                            modeDetail
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.vertical, OffriiTheme.spacingBase)
                    }
                }
            }
            .navigationTitle(String(
                format: NSLocalizedString("shareRule.title", comment: ""),
                friendName
            ))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(NSLocalizedString("common.save", comment: "")) {
                        Task { await save() }
                    }
                    .fontWeight(.semibold)
                    .disabled(isSaving)
                }
            }
            .task { await loadData() }
        }
    }

    // MARK: - Mode Selector

    @ViewBuilder
    private var modeSelector: some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            modeOption(
                mode: "all",
                icon: "list.bullet",
                title: NSLocalizedString("shareRule.all.title", comment: ""),
                subtitle: String(
                    format: NSLocalizedString("shareRule.all.subtitle", comment: ""),
                    items.filter { !$0.isPrivate && $0.isActive }.count
                )
            )

            modeOption(
                mode: "categories",
                icon: "folder",
                title: NSLocalizedString("shareRule.categories.title", comment: ""),
                subtitle: NSLocalizedString("shareRule.categories.subtitle", comment: "")
            )

            modeOption(
                mode: "selection",
                icon: "hand.tap",
                title: NSLocalizedString("shareRule.selection.title", comment: ""),
                subtitle: NSLocalizedString("shareRule.selection.subtitle", comment: "")
            )

            // Stop sharing button (only when a rule is active)
            if currentMode != "none" {
                Button {
                    selectedMode = "none"
                    Task { await save() }
                } label: {
                    Text(NSLocalizedString("shareRule.stopSharing", comment: ""))
                        .font(OffriiTypography.footnote)
                        .foregroundColor(OffriiTheme.danger)
                        .frame(maxWidth: .infinity)
                        .padding(.top, OffriiTheme.spacingSM)
                }
            }
        }
    }

    private func modeOption(mode: String, icon: String, title: String, subtitle: String) -> some View {
        Button {
            withAnimation(OffriiAnimation.snappy) { selectedMode = mode }
        } label: {
            HStack(spacing: OffriiTheme.spacingMD) {
                Image(systemName: selectedMode == mode ? "checkmark.circle.fill" : "circle")
                    .font(.system(size: 22))
                    .foregroundColor(selectedMode == mode ? OffriiTheme.primary : OffriiTheme.textMuted)

                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: OffriiTheme.spacingXS) {
                        Image(systemName: icon)
                            .font(.system(size: 14))
                            .foregroundColor(OffriiTheme.primary)
                        Text(title)
                            .font(OffriiTypography.body)
                            .fontWeight(.medium)
                            .foregroundColor(OffriiTheme.text)
                    }
                    Text(subtitle)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textSecondary)
                }

                Spacer()
            }
            .padding(OffriiTheme.spacingBase)
            .background(selectedMode == mode ? OffriiTheme.primary.opacity(0.06) : OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                    .strokeBorder(
                        selectedMode == mode ? OffriiTheme.primary : OffriiTheme.border,
                        lineWidth: selectedMode == mode ? 1.5 : 1
                    )
            )
        }
        .buttonStyle(.plain)
    }

    // MARK: - Mode Detail

    @ViewBuilder
    private var modeDetail: some View {
        if selectedMode == "all" && privateCount > 0 {
            Label(
                String(format: NSLocalizedString("shareRule.privateWarning", comment: ""), privateCount),
                systemImage: "lock.fill"
            )
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.textMuted)
            .padding(OffriiTheme.spacingSM)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(OffriiTheme.surface)
            .cornerRadius(OffriiTheme.cornerRadiusSM)
        }

        if selectedMode == "categories" {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                Text(NSLocalizedString("shareRule.selectCategories", comment: ""))
                    .font(OffriiTypography.footnote)
                    .fontWeight(.semibold)
                    .foregroundColor(OffriiTheme.textSecondary)
                    .textCase(.uppercase)

                if categories.isEmpty {
                    Text(NSLocalizedString("shareRule.noCategories", comment: ""))
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.textMuted)
                } else {
                    FlowLayout(spacing: OffriiTheme.spacingSM) {
                        ForEach(categories, id: \.id) { cat in
                            let isSelected = selectedCategoryIds.contains(cat.id)
                            let count = items.filter { $0.categoryId == cat.id && !$0.isPrivate && $0.isActive }.count
                            Button {
                                if isSelected {
                                    selectedCategoryIds.remove(cat.id)
                                } else {
                                    selectedCategoryIds.insert(cat.id)
                                }
                            } label: {
                                HStack(spacing: 4) {
                                    if let icon = cat.icon {
                                        Text(icon)
                                    }
                                    Text("\(cat.name) (\(count))")
                                        .font(OffriiTypography.footnote)
                                }
                                .padding(.horizontal, OffriiTheme.spacingSM)
                                .padding(.vertical, OffriiTheme.spacingXS)
                                .background(isSelected ? OffriiTheme.primary : OffriiTheme.surface)
                                .foregroundColor(isSelected ? .white : OffriiTheme.text)
                                .cornerRadius(OffriiTheme.cornerRadiusXL)
                            }
                        }
                    }
                }
            }
        }

        if selectedMode == "all" {
            Text(NSLocalizedString("shareRule.dynamicHint", comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.primary)
                .italic()
        }

        if selectedMode == "categories" && !selectedCategoryIds.isEmpty {
            Text(NSLocalizedString("shareRule.dynamicHint", comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.primary)
                .italic()
        }

        if selectedMode == "selection" {
            let shareable = items.filter { !$0.isPrivate && $0.isActive }
            if shareable.isEmpty {
                Text(NSLocalizedString("shareRule.noItems", comment: ""))
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.textMuted)
            } else {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                    Text(NSLocalizedString("shareRule.selectItems", comment: ""))
                        .font(OffriiTypography.footnote)
                        .fontWeight(.semibold)
                        .foregroundColor(OffriiTheme.textSecondary)
                        .textCase(.uppercase)

                    ForEach(shareable) { item in
                        let isSelected = selectedItemIds.contains(item.id)
                        Button {
                            if isSelected {
                                selectedItemIds.remove(item.id)
                            } else {
                                selectedItemIds.insert(item.id)
                            }
                        } label: {
                            HStack(spacing: OffriiTheme.spacingSM) {
                                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                                    .foregroundColor(isSelected ? OffriiTheme.primary : OffriiTheme.textMuted)
                                Text(item.name)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)
                                Spacer()
                            }
                            .padding(.vertical, OffriiTheme.spacingXS)
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
        }
    }

    // MARK: - Data

    private func loadData() async {
        isLoading = true
        async let ruleTask = CircleService.shared.getShareRule(circleId: circleId)
        async let catsTask = CategoryService.shared.listCategories()
        async let itemsTask = ItemService.shared.listItems(perPage: 200)

        do {
            let rule = try await ruleTask
            categories = (try? await catsTask) ?? []
            let itemsResponse = try? await itemsTask
            items = itemsResponse?.items ?? []

            currentMode = rule.shareMode
            selectedMode = rule.shareMode
            selectedCategoryIds = Set(rule.categoryIds)
            privateCount = items.filter { $0.isPrivate }.count

            // For selection mode, load already-shared items from circle
            if rule.shareMode == "selection" {
                let circleItems = (try? await CircleService.shared.listItems(circleId: circleId)) ?? []
                let ids = Set(circleItems.map(\.id))
                selectedItemIds = ids
                initialSharedItemIds = ids
            }
        } catch {
            currentMode = "none"
            selectedMode = "none"
        }
        isLoading = false
    }

    private func save() async {
        isSaving = true
        do {
            try await CircleService.shared.setShareRule(
                circleId: circleId,
                mode: selectedMode,
                categoryIds: selectedMode == "categories" ? Array(selectedCategoryIds) : []
            )

            // For selection mode, sync items: add new, remove deselected
            if selectedMode == "selection" {
                let toAdd = selectedItemIds.subtracting(initialSharedItemIds)
                let toRemove = initialSharedItemIds.subtracting(selectedItemIds)

                if !toAdd.isEmpty {
                    try await CircleService.shared.batchShareItems(
                        circleId: circleId,
                        itemIds: Array(toAdd)
                    )
                }
                for itemId in toRemove {
                    try? await CircleService.shared.unshareItem(
                        circleId: circleId, itemId: itemId
                    )
                }
            }

            OffriiHaptics.success()
            dismiss()
        } catch {}
        isSaving = false
    }
}

// MARK: - Flow Layout (for category chips)

private struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache _: inout ()) -> CGSize {
        let result = layout(in: proposal.width ?? 0, subviews: subviews)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal _: ProposedViewSize, subviews: Subviews, cache _: inout ()) {
        let result = layout(in: bounds.width, subviews: subviews)
        for (index, position) in result.positions.enumerated() {
            subviews[index].place(
                at: CGPoint(x: bounds.minX + position.x, y: bounds.minY + position.y),
                proposal: .unspecified
            )
        }
    }

    private func layout(in width: CGFloat, subviews: Subviews) -> (size: CGSize, positions: [CGPoint]) {
        var positions: [CGPoint] = []
        var currentX: CGFloat = 0
        var currentY: CGFloat = 0
        var lineHeight: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if currentX + size.width > width && currentX > 0 {
                currentX = 0
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            positions.append(CGPoint(x: currentX, y: currentY))
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
        }

        return (CGSize(width: width, height: currentY + lineHeight), positions)
    }
}
