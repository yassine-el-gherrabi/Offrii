import PhotosUI
import SwiftUI

// MARK: - OffriiImagePicker

struct OffriiImagePicker: View {
    @Binding var selectedImage: UIImage?
    var existingImageUrl: URL?
    var isUploading: Bool = false
    @State private var pickerItem: PhotosPickerItem?
    @State private var showSourceSheet = false
    @State private var showCamera = false

    private var hasImage: Bool {
        selectedImage != nil || existingImageUrl != nil
    }

    private var cameraAvailable: Bool {
        UIImagePickerController.isSourceTypeAvailable(.camera)
    }

    var body: some View {
        ZStack {
            // Image or placeholder
            if let selectedImage {
                Image(uiImage: selectedImage)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .frame(maxWidth: .infinity)
                    .frame(height: 180)
                    .clipped()
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
            } else if let existingImageUrl {
                AsyncImage(url: existingImageUrl) { phase in
                    switch phase {
                    case .success(let image):
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(maxWidth: .infinity)
                            .frame(height: 180)
                            .clipped()
                            .cornerRadius(OffriiTheme.cornerRadiusMD)
                    default:
                        placeholderView
                    }
                }
            } else {
                placeholderView
            }

            // Upload progress overlay
            if isUploading {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .fill(.black.opacity(0.4))
                    .frame(maxWidth: .infinity)
                    .frame(height: hasImage ? 180 : 100)
                    .overlay {
                        ProgressView()
                            .tint(.white)
                            .scaleEffect(1.2)
                    }
            }

            // Overlay controls
            if hasImage && !isUploading {
                VStack {
                    HStack {
                        Spacer()
                        // Remove button
                        Button {
                            withAnimation(OffriiAnimation.snappy) {
                                selectedImage = nil
                                pickerItem = nil
                            }
                        } label: {
                            Image(systemName: "xmark")
                                .font(.system(size: 12, weight: .bold))
                                .foregroundColor(.white)
                                .frame(width: 28, height: 28)
                                .background(.black.opacity(0.4))
                                .clipShape(Circle())
                                .overlay(Circle().strokeBorder(.white.opacity(0.2), lineWidth: 0.5))
                        }
                    }
                    .padding(OffriiTheme.spacingSM)

                    Spacer()

                    // Change photo button
                    Button {
                        showSourceSheet = true
                    } label: {
                        HStack(spacing: 6) {
                            Image(systemName: "camera.fill")
                                .font(.system(size: 12))
                            Text(NSLocalizedString("imagePicker.change", comment: ""))
                                .font(.system(size: 12, weight: .medium))
                        }
                        .foregroundColor(.white)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(.black.opacity(0.4))
                        .clipShape(Capsule())
                        .overlay(Capsule().strokeBorder(.white.opacity(0.2), lineWidth: 0.5))
                    }
                    .padding(.bottom, OffriiTheme.spacingSM)
                }
            }
        }
        .frame(maxWidth: .infinity)
        .frame(height: hasImage ? 180 : nil)
        .confirmationDialog(
            NSLocalizedString("imagePicker.add", comment: ""),
            isPresented: $showSourceSheet,
            titleVisibility: .visible
        ) {
            if cameraAvailable {
                Button(NSLocalizedString("imagePicker.takePhoto", comment: "")) {
                    showCamera = true
                }
            }
            // PhotosPicker can't be triggered programmatically, so we use a hidden one
            // and instead trigger it via pickerItem change
            Button(NSLocalizedString("imagePicker.chooseFromGallery", comment: "")) {
                // Small delay to let the sheet dismiss before opening picker
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                    triggerPhotoPicker = true
                }
            }
        }
        .fullScreenCover(isPresented: $showCamera) {
            CameraImagePicker(image: $selectedImage)
                .ignoresSafeArea()
        }
        .onChange(of: pickerItem) { _, newItem in
            Task {
                if let data = try? await newItem?.loadTransferable(type: Data.self),
                   let uiImage = UIImage(data: data) {
                    withAnimation(OffriiAnimation.defaultSpring) {
                        selectedImage = uiImage
                    }
                }
            }
        }
        .photosPicker(
            isPresented: $triggerPhotoPicker,
            selection: $pickerItem,
            matching: .images
        )
    }

    @State private var triggerPhotoPicker = false

    // Placeholder — only shown when no image
    private var placeholderView: some View {
        Button {
            showSourceSheet = true
        } label: {
            VStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: "camera.fill")
                    .font(.system(size: 24))
                    .foregroundColor(OffriiTheme.textMuted)

                Text(NSLocalizedString("imagePicker.add", comment: ""))
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(OffriiTheme.textMuted)
            }
            .frame(maxWidth: .infinity)
            .frame(height: 100)
            .background(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .strokeBorder(OffriiTheme.border, style: StrokeStyle(lineWidth: 1.5, dash: [6]))
            )
        }
    }
}
