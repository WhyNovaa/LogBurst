fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Используем configure() для настройки
    let mut config = tonic_build::configure();

    // Можно указать опцию для настройки внешних типов,
    // чтобы prost-types::Timestamp автоматически маппился на
    // google.protobuf.Timestamp, но обычно это не обязательно.

    // Используем compile_protos, который принимает путь к proto-файлу
    // и список директорий, где искать импорты (например, для Timestamp).
    config.compile_protos(
        &["proto/log.proto"], // Путь к вашему основному файлу
        &["proto"],        // Папка, где находится log.proto и откуда он ищмпортирует
    )?;

    Ok(())
}