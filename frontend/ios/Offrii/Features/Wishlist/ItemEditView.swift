import SwiftUI

struct ItemEditView: View {
    let item: Item
    let onSave: (Item) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var name: String
    @State private var description: String
    @State private var url: String
    @State private var estimatedPrice: String
    @State private var priority: Int
    @State private var categoryId: UUID?
    @State private var isSaving = false
    @State private var categories: [CategoryResponse] = []
    @State private var showCategoryPicker = false

    init(item: Item, onSave: @escaping (Item) -> Void) {
        self.item = item
        self.onSave = onSave
        _name = State(initialValue: item.name)
        _description = State(initialValue: item.description ?? "")
        _url = State(initialValue: item.url ?? "")
        _estimatedPrice = State(initialValue: item.estimatedPrice.map { "\($0)" } ?? "")
        _priority = State(initialValue: item.priority)
        _categoryId = State(initialValue: item.categoryId)
    }

    var body: some View {
        ScrollView {
            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiCard {
                    VStack(spacing: OffriiTheme.spacingMD) {
                        OffriiTextField(
                            label: NSLocalizedString("item.name", comment: ""),
                            text: $name,
                            placeholder: NSLocalizedString("item.name", comment: ""),
                            autocapitalization: .sentences
                        )

                        OffriiTextField(
                            label: NSLocalizedString("item.description", comment: ""),
                            text: $description,
                            placeholder: NSLocalizedString("item.description", comment: ""),
                            autocapitalization: .sentences
                        )

                        OffriiTextField(
                            label: NSLocalizedString("item.url", comment: ""),
                            text: $url,
                            placeholder: "https://",
                            keyboardType: .URL,
                            autocapitalization: .never
                        )

                        OffriiTextField(
                            label: NSLocalizedString("item.estimatedPrice", comment: ""),
                            text: $estimatedPrice,
                            placeholder: "0.00",
                            keyboardType: .decimalPad
                        )

                        // Priority picker
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            Text(NSLocalizedString("item.priority", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.textSecondary)

                            Picker("", selection: $priority) {
                                Text(NSLocalizedString("priority.low", comment: "")).tag(1)
                                Text(NSLocalizedString("priority.medium", comment: "")).tag(2)
                                Text(NSLocalizedString("priority.high", comment: "")).tag(3)
                            }
                            .pickerStyle(.segmented)
                        }

                        // Category
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            Text(NSLocalizedString("item.category", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.textSecondary)

                            Button {
                                showCategoryPicker = true
                            } label: {
                                HStack {
                                    Text(categoryLabel)
                                        .font(OffriiTypography.body)
                                        .foregroundColor(categoryId == nil ? OffriiTheme.textMuted : OffriiTheme.text)
                                    Spacer()
                                    Image(systemName: "chevron.right")
                                        .font(.system(size: 12))
                                        .foregroundColor(OffriiTheme.textMuted)
                                }
                                .padding(.horizontal, OffriiTheme.spacingMD)
                                .padding(.vertical, 14)
                                .background(OffriiTheme.card)
                                .cornerRadius(OffriiTheme.cornerRadiusMD)
                                .overlay(
                                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                                        .strokeBorder(OffriiTheme.border, lineWidth: 1)
                                )
                            }
                        }
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)

                OffriiButton(
                    NSLocalizedString("item.save", comment: ""),
                    isLoading: isSaving,
                    isDisabled: name.trimmingCharacters(in: .whitespaces).isEmpty
                ) {
                    Task { await save() }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
            }
            .padding(.top, OffriiTheme.spacingMD)
        }
        .background(OffriiTheme.cardSurface.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("wishlist.edit", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button(NSLocalizedString("common.cancel", comment: "")) {
                    dismiss()
                }
                .foregroundColor(OffriiTheme.primary)
            }
        }
        .sheet(isPresented: $showCategoryPicker) {
            CategoryPickerView(
                categories: categories,
                selectedId: $categoryId
            )
        }
        .task {
            do {
                categories = try await CategoryService.shared.listCategories()
            } catch {}
        }
    }

    private var categoryLabel: String {
        if let id = categoryId, let cat = categories.first(where: { $0.id == id }) {
            return cat.name
        }
        return NSLocalizedString("item.category", comment: "")
    }

    private func save() async {
        isSaving = true
        let trimmedName = name.trimmingCharacters(in: .whitespaces)
        let trimmedDesc = description.trimmingCharacters(in: .whitespaces)
        let trimmedUrl = url.trimmingCharacters(in: .whitespaces)
        let price = Decimal(string: estimatedPrice)

        do {
            let updated = try await ItemService.shared.updateItem(
                id: item.id,
                name: trimmedName,
                description: trimmedDesc.isEmpty ? nil : trimmedDesc,
                url: trimmedUrl.isEmpty ? nil : trimmedUrl,
                estimatedPrice: price,
                priority: priority,
                categoryId: categoryId
            )
            onSave(updated)
            dismiss()
        } catch {
            // Could show error
        }
        isSaving = false
    }
}
